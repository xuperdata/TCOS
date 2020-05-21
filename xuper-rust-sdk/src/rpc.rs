/// 负责通信，运行在TEE之外
use std::env;
use std::sync::Arc;

use futures::executor;

use grpc::ClientStub;
use grpc::ClientStubExt;

use crate::protos::xchain;
use crate::protos::xendorser;
use crate::protos::xendorser_grpc;
use crate::config;

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
    pub xchain_ser: String,
    pub contract_account: String,
    client: grpc::client::ClientStub,
    address: String,
}

impl Session {
    fn new(host: &str, port: u16) -> Self {
        //TODO: 设置超时，以及body大小
        let conn = Arc::new(grpc::ClientBuilder::new::new(host, port).build().unwrap());
        let client = xendorser_grpc::xendorserClient::with_client(conn);
        let s: Session = ::std::default::Default::default();
        s.client = grpc_client;
        s
    }

    fn call(&self, r: xendorser::EndorserRequest) -> Result<xendorser::EndorserResponse> {
        let resp = self
            .cleint
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
        let mut tx_inputs = std::vec::Vec<xchain::TxInput>::new();
        for utxo in utxo_output.utxoList {
            let ti =  xchain::TxInput {
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
        return Ok((tx_inputs, to))
    }

    fn generate_tx_output(&self, to: &String, amount: &String, fee: &str) -> Result<(Vec<xchain::TxOutput>)> {
        let tx_outputs = std::vec::Vec<xchain::TxOutput>::new();
        if !to.is_empty() && amount.as_str() != "0" {
            let t = xchain::TxOutput {
                address: to,
                amount: amount,
                frozen_height: 0,
            };
            tx_outputs.push(t);
        }
        if !fee.is_empty() && fee != "0" {
            let t xchain::TxOutput{
                address: "$",
                amount: fee,
                frozen_height: 0,
            }
            tx_outputs.push(t);
        }
        Ok(tx_outputs)
    }


    pub fn gen_compliance_check_tx(
        &self,
        resp: &mut xchain::PreExecWithSelectUTXOResponse,
    ) -> Result<xchain::Transaction> {
        let total_need = num_bigint::BigInt::from_str(self.cfg.compliance_check.compliance_check_endorse_service_fee)?;
        let (tx_inputs, tx_output) = self.generate_tx_input(resp.get_utxoOutput(), utxo_total)?;
        let mut tx_outputs = self.generate_tx_output(
            self.cfg.compliance_check.compliance_check_endorse_service_fee_addr,
            self.cfg.compliance_check.compliance_check_endorse_service_fee,
            "0",
        )?;

        if !tx_output.address.is_empty() {
            tx_outputs.push(tx_output);
        }

        // compose transaction
    }
}
