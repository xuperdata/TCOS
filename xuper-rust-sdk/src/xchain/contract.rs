use crate::errors::{Error, ErrorKind, Result};
use crate::{rpc, wallet, consts, config, protos::xchain};

/// account在chain上面给to转账amount，小费是fee，留言是desc
pub fn invoke_contract(
    account: &wallet::Account,
    chain: &rpc::ChainClient,
    method_name: &String,
    args: &std::collections::HashMap,
) -> Result<String> {
    let invoke_req = xchain::InvokeRequest {
        module_name: String::from("wasm"),
        contract_name: account.contract_name.to_owned(),
        method_name: method_name.to_owned(),
        args: args,
        resource_limits: xchain::ResourceLimit::new(),
        amount: String::from("0"),
    };
    let invoke_requests = vec![invoke_req; 1];
    let mut auth_requires = vec[String; 0];
    if !account.contract_account.empty() {
        let mut s = account.contract_account.to_owned();
        s.push_str("/");
        s.push_str(account.address.to_owned());
        auth_requires.push(s);
    };
    auth_requires.push(config::CONFIG.compliance_check_endorse_service_addr.to_owned());

    let invoke_rpc_request = xchain::InvokeRPCRequest {
        header: xchain::Header::new(),
        bcname: chain.chain_name.to_owned(),
        requests: invoke_requests,
        initiator: account.address.to_owned(),
        auth_require: auth_requires,
    };

    let total_amount = consts::str_as_i64(config::CONFIG.compliance_check_endorse_service_fee.as_str());
    let pre_sel_utxo_req = xchain::PreExecWithSelectUTXORequest {
        bcname: chain.chain_name.to_owned(),
        address: account.address.to_owned(),
        total_amount: total_amount,
        request: invoke_rpc_request,
    };

    let mut msg = rpc::Message {
        to: String::from(""),
        fee: String::from("0"),
        desc: String::from(""),
        pre_sel_utxo_req: pre_sel_utxo_req,
        invoke_rpc_req: invoke_rpc_request,
        auth_require: auth_requires,
        amount: String::from("0"),
        frozen_height: 0,
        initiator: account.address.to_owned(),
    };

    let sess = rpc::Session::new(account, chain, msg);
    let resp = sess.pre_exec_with_select_utxo()?;

    //TODO clear pre_sel_utxo_req & invoke_rpc_req
    msg.pre_sel_utxo_req = xchain::PreExecWithSelectUTXORequest::new();
    msg.invoke_rpc_req = xchain::InvokeRPCRequest::new();
    msg.fee = resp.response.gas_used.to_string();
    sess.gen_complete_tx_and_post(resp)
}

pub fn query_contract (account:&wallet::Account, client: &rpc::ChainClient,
                        method_name: &String,
                       args: &std::collections::HashMap) -> Result<xchain::InvokeResponse> {

    let invoke_req = xchain::InvokeRequest {
        module_name: String::from("wasm"),
        contract_name: account.contract_name.to_owned(),
        method_name: method_name.to_owned(),
        args: args,
        resource_limits: xchain::ResourceLimit::new(),
        amount: String::from("0"),
    };
    let invoke_requests = vec![invoke_req; 1];
    let mut auth_requires = vec[String; 0];
    if !account.contract_account.empty() {
        let mut s = account.contract_account.to_owned();
        s.push_str("/");
        s.push_str(account.address.to_owned());
        auth_requires.push(s);
    };
    auth_requires.push(config::CONFIG.compliance_check_endorse_service_addr.to_owned());

    let invoke_rpc_request = xchain::InvokeRPCRequest {
        header: xchain::Header::new(),
        bcname: chain.chain_name.to_owned(),
        requests: invoke_requests,
        initiator: account.address.to_owned(),
        auth_require: auth_requires,
    };

    let mut msg = rpc::Message {
        to: String::from(""),
        fee: String::from("0"),
        desc: String::from(""),
        pre_sel_utxo_req: xchain::PreExecWithSelectUTXORequest::new(),
        invoke_rpc_req: invoke_rpc_request,
        auth_require: auth_requires,
        amount: String::from("0"),
        frozen_height: 0,
        initiator: account.address.to_owned(),
    };

    let sess = rpc::Session::new(account, chain, msg);
    sess.pre_exec()
}
