/// 负责通信，运行在TEE之外
use std::env;
use std::sync::Arc;

use futures::executor;

use grpc::ClientStub;
use grpc::ClientStubExt;

use crate::config;
use crate::protos::xchain;
use crate::protos::xendorser;
use crate::protos::xendorser_grpc;

use xchain_crypto::errors::{Error, ErrorKind, Result};

pub struct Session {
    pub cfg: config::CommConfig,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub fee: String,
    pub desc: String,
    pub frozen_height: i64,
    pub invoke_rpc_req: xchain::InvokeRPCRequest,
    pub pre_sel_utxo_req: xchain::PreExecWithSelectUTXORequest,
    pub initiator: String,
    pub auth_require: Vec<String>,
    pub chain_name: String,
    pub contract_account: String,
    client_endorser: grpc::client::ClientStub,
    client_xchain: grpc::client::ClientStub,
    address: String,
}

impl Session {
    fn new(host: &str, port: u16, port_xchain: u16) -> Self {
        //TODO: 设置超时，以及body大小
        let conn = Arc::new(grpc::ClientBuilder::new::new(host, port).build().unwrap());
        let grpc_client = xendorser_grpc::xendorserClient::with_client(conn);
        let s: Session = ::std::default::Default::default();
        s.client_endorser = grpc_client;
        let conn = Arc::new(
            grpc::ClientBuilder::new::new(host, port_xchain)
                .build()
                .unwrap(),
        );
        let grpc_client = xendorser_grpc::xendorserClient::with_client(conn);
        s.client_xchain = grpc_client;
        s
    }

    fn call(&self, r: xendorser::EndorserRequest) -> Result<xendorser::EndorserResponse> {
        let resp = self
            .client_endorser
            .endorser_call(grpc::RequestOptions::new(), endorser_request)
            .join_metadata_result();
        executor::block_on(resp)
    }
    fn check_resp_code(resp: &[xchain::ContractResponse]) -> Result<()> {
        for i in resp.iter() {
            if i.status > 400 {
                return Err(Error::from(ErrorKind::KeyParamNotMatchError));
            }
        }
        Ok(())
    }

    pub fn pre_exec_with_select_utxo(&self) -> Result<xchain::PreExecWithSelectUTXOResponse> {
        let request_data = serde_json::to_string(self.pre_sel_utxo_req)?;
        let endorser_request = xendorser::EndorserRequest::new();
        endorser_request.set_RequestName("PreExecWithFee");
        endorser_request.set_BcName(self.chain_name);
        endorser_request.set_RequestData(request_data);
        let resp = self.call(endorser_request)?;

        let pre_exec_with_select_utxo_resp: xchain::PreExecWithSelectUTXOResponse =
            serde_json::from_str(resp.ResponseData)?;
        self.check_resp_code(
            pre_exec_with_select_utxo_resp
                .get_response()
                .get_responses(),
        )?;
        Ok(pre_exec_with_select_utxo_resp)
    }

    fn generate_tx_input(
        &self,
        utxo_output: &xchain::UtxoOutput,
        total_need: &num_bigint::BigInt,
    ) -> Result<(Vec<xchain::TxInput>, xchain::TxOutput)> {
        let mut tx_inputs = std::vec::Vec::<xchain::TxInput>::new();
        for utxo in utxo_output.utxoList {
            let ti = xchain::TxInput {
                ref_txid: utxo.ref_txid,
                ref_offset: utxo.ref_offset,
                from_addr: utxo.to_addr,
                amount: utxo.amount,
            };
            tx_inputs.push(ti);
        }

        let utxo_total = num_bigint::BigInt::from_str(utxo_output.totalSelected)?;

        let mut to = xchain::TxOutput::new();
        if utxo_total.cmp(total_need) == num_bigint::BigInt::Order::Greater {
            let delta = utxo_total.Sub(total_need);
            to.to_addr = self.address;
            to.amount = delta.to_bytes_be();
        }
        return Ok((tx_inputs, to));
    }

    fn generate_tx_output(
        &self,
        to: &String,
        amount: &String,
        fee: &str,
    ) -> Result<(Vec<xchain::TxOutput>)> {
        let tx_outputs = std::vec::Vec::<xchain::TxOutput>::new();
        if !to.is_empty() && amount.as_str() != "0" {
            let t = xchain::TxOutput {
                address: to,
                amount: amount,
                frozen_height: 0,
            };
            tx_outputs.push(t);
        }
        if !fee.is_empty() && fee != "0" {
            let t = xchain::TxOutput {
                address: "$",
                amount: fee,
                frozen_height: 0,
            };
            tx_outputs.push(t);
        }
        Ok(tx_outputs)
    }

    pub fn gen_compliance_check_tx(
        &self,
        resp: &mut xchain::PreExecWithSelectUTXOResponse,
    ) -> Result<xchain::Transaction> {
        let total_need = num_bigint::BigInt::from_str(
            self.cfg
                .compliance_check
                .compliance_check_endorse_service_fee,
        )?;
        let (tx_inputs, tx_output) = self.generate_tx_input(resp.get_utxoOutput(), utxo_total)?;
        let mut tx_outputs = self.generate_tx_output(
            self.cfg
                .compliance_check
                .compliance_check_endorse_service_fee_addr,
            self.cfg
                .compliance_check
                .compliance_check_endorse_service_fee,
            "0",
        )?;

        if !tx_output.address.is_empty() {
            tx_outputs.push(tx_output);
        }

        // compose transaction
        let tx = xchain::Transaction::new();
        tx.set_desc(vec![]);
        tx.set_version(super::consts::TxVersion);
        tx.set_coinbase(true);
        tx.set_timestamp(super::consts::now_as_nanos());
        tx.set_tx_inputs(tx_inputs);
        tx.set_tx_outputs(tx_outputs);
        tx.set_initiator(self.initiator);
        tx.set_nonce(super::wallet::get_nonce()?);

        let digest_hash = super::wallet::make_tx_digest_hash(tx)?;

        //TODO sign the digest_hash

        tx.set_txid(super::wallet::make_transaction_id(tx)?);
        Ok(tx)
    }

    pub fn gen_real_tx(
        &self,
        resp: &xchain::PreExecWithSelectUTXOResponse,
        cctx: &xchain::Transaction,
    ) -> Result<xchain::Transaction> {
        let tx_outputs = self.generate_tx_output(self.to, self.amount, self.fee)?;

        let mut total_selected = num_bigint::BigInt::new(0);
        let mut utxo_list = std::vec::Vec::<xchain::Utxo>::new();
        for (index, tx_output) in cctx.tx_outputs.iter() {
            if tx_output.to_addr == self.initiator {
                let t = xchain::Utxo {
                    amount: tx_output.amount,
                    to_addr: tx_output.to_addr,
                    ref_txid: cctx.txid,
                    ref_offset: index,
                };
                utxo_list.push(t);
                let um = num_bigint::BigInt::from_slice(num_bigint::Sign::Plus, &tx_output.amount)?;
                total_selected.add_assign(um);
            };
        }
        let utxo_output = xchain::UtxoOutput {
            utxoOutput: utxo_list,
            total_selected: total_selected.to_string(),
        };

        let mut total_need = num_bigint::BigInt::from_slice(num_bigint::Sign::Plus, &self.amount)?;
        total_need.add_assign(num_bigint::BigInt::from_slice(
            num_bigint::Sign::Plus,
            &self.fee,
        )?)?;

        let (tx_inputs, delta_tx_ouput) = self.generate_tx_input(utxo_output, total_need)?;
        if !delta_tx_ouput.address.is_empty() {
            tx_outputs.push(tx_output);
        }

        let tx = xchain::Transaction::new();
        tx.set_desc(vec![]);
        tx.set_version(super::consts::TxVersion);
        tx.set_coinbase(false);
        tx.set_timestamp(super::consts::now_as_nanos());
        tx.set_tx_inputs(tx_inputs);
        tx.set_tx_outputs(tx_outputs);
        tx.set_initiator(self.initiator);
        tx.set_nonce(super::wallet::get_nonce()?);

        //TODO auth require

        let digest_hash = super::wallet::make_tx_digest_hash(tx)?;

        //TODO sign the digest_hash

        tx.set_txid(super::wallet::make_transaction_id(tx)?);
        Ok(tx)
    }

    pub fn compliance_check(
        &self,
        tx: &xchain::Transaction,
        fee: &xchain::Transaction,
    ) -> Result<xchain::SignatureInfo> {
        let tx_status = xchain::TxStatus {
            bcname: xc.chain_name,
            tx: tx,
        };
        let request_data = serde_json::to_string(tx_status)?;
        let endorser_request = xendorser::EndorserRequest::new();
        endorser_request.set_RequestName("ComplianceCheck");
        endorser_request.set_BcName(self.chain_name);
        endorser_request.set_RequestData(request_data);
        let resp = self.call(endorser_request)?;
        Ok(resp.get_EndorserSign())
    }

    pub fn post_tx(&self, tx: &xchain::Transaction) -> Result<()> {
        let tx_status = xchain::TxStatus {
            bcname: xc.chain_name,
            status: xchain::TransactionStatus::UNCONFIRM,
            tx: tx,
            txid: tx.txid,
        };
        let resp = self
            .client_xchain
            .query_tx(grpc::RequestOptions::new(), tx_status)
            .join_metadata_result();
        executor::block_on(resp);
        if resp.Header.Error != xchain::XChainErrorEnum::SUCCESS {
            println!("post tx failed");
            return Err(Error::from(ErrorKind::ParseError));
        }
        Ok(())
    }

    pub fn gen_complete_tx_and_post(
        &self,
        pre_exec_resp: &xchain::PreExecWithSelectUTXOResponse,
    ) -> Result<String> {
        let cctx = self.gen_compliance_check_tx(pre_exec_resp)?;
        let mut tx = self.gen_real_tx(pre_exec_resp, cctx)?;
        let end_sign = self.compliance_check(tx, cctx);

        tx.auth_require_signs.push(end_sign);
        tx.set_txid(super::wallet::make_transaction_id(tx));
        self.post_tx(tx)?;
        Ok(hex::encode(tx.txid))
    }
    pub fn query_tx(&self, txid: &String) -> Result<xchain::TxStatus> {
        let tx_status = xchain::TxStatus {
            bcname: self.chain_name,
            txid: hex::decode(txid),
        };
        let resp = self
            .client_xchain
            .query_tx(grpc::RequestOptions::new(), tx_status)
            .join_metadata_result();
        executor::block_on(resp);

        if resp.Header.Error != xchain::XChainErrorEnum::SUCCESS {
            println!("query tx failed");
            return Err(Error::from(ErrorKind::ParseError));
        }
        // TODO check txid if null
        Ok(resp)
    }

    pub fn pre_exec(&self) -> Result<xchain::InvokeRPCResponse> {
        let resp = self
            .client_xchain
            .pre_exec(grpc::RequestOptions::new(), self.invoke_rpc_req)
            .join_metadata_result();
        executor::block_on(resp);
        self.check_resp_code(resp.get_response().get_responses())?;
        Ok(resp)
    }
}
