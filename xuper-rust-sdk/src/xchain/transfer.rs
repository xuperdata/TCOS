use crate::{
    config, consts,
    errors::{Error, ErrorKind, Result},
    protos::xchain,
    rpc, wallet,
};

/// account在chain上面给to转账amount，小费是fee，留言是desc
pub fn transfer(
    account: &wallet::Account,
    chain: &rpc::ChainClient,
    to: &String,
    amount: &String,
    fee: &String,
    desc: &String,
) -> Result<String> {
    let amount_bk = amount.to_owned();
    let amount = consts::str_as_i64(amount.as_str())?;
    let fee = consts::str_as_i64(fee.as_str())?;
    let auth_requires = vec![
        config::CONFIG
            .compliance_check_endorse_service_addr
            .to_owned();
        1
    ];

    // make Header
    let header = xchain::Header::new();
    let endorser_fee =
        consts::str_as_i64(config::CONFIG.compliance_check_endorse_service_fee.as_str());
    if endorser_fee > amount {
        return Err(Error::from(ErrorKind::InvalidArguments));
    }
    let total_amount = amount + fee + endorser_fee;
    //防止溢出
    if total_amount < amount {
        return Err(Error::from(ErrorKind::InvalidArguments));
    }

    let invoke_rpc_request = xchain::InvokeRPCRequest {
        header: header,
        bcname: chain.chain_name.to_owned(),
        requests: vec![],
        initiator: account.address.to_owned(),
        auth_require: auth_requires,
    };

    let pre_sel_utxo_req = xchain::PreExecWithSelectUTXORequest {
        bcname: chain.chain_name.to_owned(),
        address: account.address.to_owned(),
        total_amount: total_amount,
        request: invoke_rpc_request,
    };

    let msg = rpc::Message {
        to: to.to_owned(),
        fee: fee.to_owned(),
        desc: desc.to_owned(),
        pre_sel_utxo_req: pre_sel_utxo_req,
        invoke_rpc_req: invoke_rpc_request,
        auth_require: auth_requires,
        amount: amount_bk,
        frozen_height: 0,
        initiator: account.address.to_owned(),
    };

    let sess = rpc::Session::new(chain, account, msg);
    let pre_exe_with_sel_res = sess.pre_exec_with_select_utxo()?;
    sess.gen_complete_tx_and_post(pre_exe_with_sel_res)
}
