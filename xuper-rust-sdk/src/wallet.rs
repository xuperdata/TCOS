/// 保管私钥，提供签名和验签
/// 要在TEE里面运行
use serde::ser::{Serialize, SerializeStruct, Serializer};
use xchain_crypto::errors::*;

use crate::protos::xchain;

pub fn set_seed() -> Result<()> {
    Ok(())
}

pub fn encode_tx(tx: &xchain::Transaction, has_signs: bool) -> Result<Vec<u8>> {}

impl Serialize for xchain::Transaction {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {

    }
}

pub fn make_tx_digest_hash(tx: &xchain::Transaction) -> Result<Vec<u8>> {
    let d = encode_tx(tx, false)?;
    xchain_crypto::hash::double_sha256(d)
}

pub fn make_transaction_id(tx: &xchain::Transaction) -> Result<Vec<u8>> {
    let d = encode_tx(tx, true)?;
    xchain_crypto::hash::double_sha256(d)
}

pub fn sign() {}
