use crate::errors::Result;
use crate::{config, protos::xchain, rpc, wallet};

/// account在chain上面给to转账amount，小费是fee，留言是desc
pub fn invoke_contract(
    account: &wallet::Account,
    chain: &rpc::ChainClient,
    method_name: &String,
    args: std::collections::HashMap<String, Vec<u8>>,
) -> Result<String> {
    let mut invoke_req = xchain::InvokeRequest::new();
    invoke_req.set_module_name(String::from("wasm"));
    invoke_req.set_contract_name(account.contract_name.to_owned());
    invoke_req.set_method_name(method_name.to_owned());
    invoke_req.set_args(args);
    invoke_req.set_amount(String::from("0"));

    let invoke_requests = vec![invoke_req; 1];
    let mut auth_requires = vec![];
    if !account.contract_account.is_empty() {
        let mut s = account.contract_account.to_owned();
        s.push_str("/");
        s.push_str(account.address.to_owned().as_str());
        auth_requires.push(s);
    };
    auth_requires.push(
        config::CONFIG
            .read()
            .unwrap()
            .compliance_check
            .compliance_check_endorse_service_addr
            .to_owned(),
    );

    let mut invoke_rpc_request = xchain::InvokeRPCRequest::new();
    invoke_rpc_request.set_bcname(chain.chain_name.to_owned());
    invoke_rpc_request.set_requests(protobuf::RepeatedField::from_vec(invoke_requests));
    invoke_rpc_request.set_initiator(account.address.to_owned());
    invoke_rpc_request.set_auth_require(protobuf::RepeatedField::from_vec(auth_requires.clone()));

    let total_amount = config::CONFIG
        .read()
        .unwrap()
        .compliance_check
        .compliance_check_endorse_service_fee;

    let mut pre_sel_utxo_req = xchain::PreExecWithSelectUTXORequest::new();
    pre_sel_utxo_req.set_bcname(chain.chain_name.to_owned());
    pre_sel_utxo_req.set_address(account.address.to_owned());
    pre_sel_utxo_req.set_totalAmount(total_amount as i64);
    pre_sel_utxo_req.set_request(invoke_rpc_request.clone());

    let msg = rpc::Message {
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

    let sess = rpc::Session::new(chain, account, &msg);
    let mut resp = sess.pre_exec_with_select_utxo()?;

    /*  TODO clean some fields
    msg.pre_sel_utxo_req = xchain::PreExecWithSelectUTXORequest::new();
    msg.invoke_rpc_req = xchain::InvokeRPCRequest::new();
    msg.fee = resp.response.unwrap().gas_used.to_string();
    */
    sess.gen_complete_tx_and_post(&mut resp)
}

pub fn query_contract(
    account: &wallet::Account,
    client: &rpc::ChainClient,
    method_name: &String,
    args: std::collections::HashMap<String, Vec<u8>>,
) -> Result<xchain::InvokeRPCResponse> {
    let mut invoke_req = xchain::InvokeRequest::new();
    invoke_req.set_module_name(String::from("wasm"));
    invoke_req.set_contract_name(account.contract_name.to_owned());
    invoke_req.set_method_name(method_name.to_owned());
    invoke_req.set_args(args);
    invoke_req.set_amount(String::from("0"));
    let invoke_requests = vec![invoke_req; 1];
    let mut auth_requires = vec![];
    if !account.contract_account.is_empty() {
        let mut s = account.contract_account.to_owned();
        s.push_str("/");
        s.push_str(account.address.to_owned().as_str());
        auth_requires.push(s);
    };
    auth_requires.push(
        config::CONFIG
            .read()
            .unwrap()
            .compliance_check
            .compliance_check_endorse_service_addr
            .to_owned(),
    );

    let mut invoke_rpc_request = xchain::InvokeRPCRequest::new();
    invoke_rpc_request.set_bcname(client.chain_name.to_owned());
    invoke_rpc_request.set_requests(protobuf::RepeatedField::from_vec(invoke_requests));
    invoke_rpc_request.set_initiator(account.address.to_owned());
    invoke_rpc_request.set_auth_require(protobuf::RepeatedField::from_vec(auth_requires.clone()));

    let msg = rpc::Message {
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

    let sess = rpc::Session::new(client, account, &msg);
    sess.pre_exec()
}