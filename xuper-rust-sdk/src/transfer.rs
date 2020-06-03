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
            .read()
            .unwrap()
            .compliance_check
            .compliance_check_endorse_service_addr
            .to_owned();
        1
    ];

    let endorser_fee = config::CONFIG
        .read()
        .unwrap()
        .compliance_check
        .compliance_check_endorse_service_fee as i64;
    // TODO 应该不用判断
    if endorser_fee > amount {
        println!("endorser_fee should smaller than amount");
        return Err(Error::from(ErrorKind::InvalidArguments));
    }
    let total_amount = amount + fee + endorser_fee;
    //防止溢出
    if total_amount < amount {
        println!("totoal_amount should be greater than amount");
        return Err(Error::from(ErrorKind::InvalidArguments));
    }

    let mut invoke_rpc_request = xchain::InvokeRPCRequest::new();
    invoke_rpc_request.set_bcname(chain.chain_name.to_owned());
    invoke_rpc_request.set_requests(protobuf::RepeatedField::from_vec(vec![]));
    invoke_rpc_request.set_initiator(account.address.to_owned());
    invoke_rpc_request.set_auth_require(protobuf::RepeatedField::from_vec(auth_requires.clone()));

    let mut pre_sel_utxo_req = xchain::PreExecWithSelectUTXORequest::new();
    pre_sel_utxo_req.set_bcname(chain.chain_name.to_owned());
    pre_sel_utxo_req.set_address(account.address.to_owned());
    pre_sel_utxo_req.set_totalAmount(total_amount);
    pre_sel_utxo_req.set_request(invoke_rpc_request.clone());

    let msg = rpc::Message {
        to: to.to_owned(),
        fee: fee.to_string(),
        desc: desc.to_owned(),
        pre_sel_utxo_req: pre_sel_utxo_req,
        invoke_rpc_req: invoke_rpc_request,
        auth_require: auth_requires,
        amount: amount_bk,
        frozen_height: 0,
        initiator: account.address.to_owned(),
    };

    let sess = rpc::Session::new(chain, account, &msg);
    let mut pre_exe_with_sel_res = sess.pre_exec_with_select_utxo()?;
    sess.gen_complete_tx_and_post(&mut pre_exe_with_sel_res)
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    #[test]
    fn test_transfer() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("key/private.key");
        let acc = super::wallet::Account::new(
            d.to_str().unwrap(),
            Default::default(),
            "XC1111111111000000@xuper",
        );
        let to = "dpzuVdosQrF2kmzumhVeFQZa1aYcdgFpN".to_string();
        let bcname = String::from("xuper");
        let chain = super::rpc::ChainClient::new(&bcname);
        let amount = "1401".to_string();
        let fee = "0".to_string();
        let desc = "test duanbing".to_string();

        let res = super::transfer(&acc, &chain, &to, &amount, &fee, &desc);
        println!("transfer res: {:?}", res);
        assert_eq!(res.is_ok(), true);
        let txid = res.unwrap();
        println!("txid: {:?}", txid);

        let msg: crate::rpc::Message = Default::default();
        let sess = crate::rpc::Session::new(&chain, &acc, &msg);
        let res = sess.query_tx(&txid);
        assert_eq!(res.is_ok(), true);
        println!("{:?}", res.unwrap());
    }
}
