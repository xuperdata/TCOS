use crate::errors::*;
use crate::protos::xchain;
use rand::rngs::StdRng;
use rand_core::{RngCore, SeedableRng};
/// 保管私钥，提供签名和验签
/// 要在TEE里面运行
/// 唯一可以调用xchain_crypto的地方
use serde::{Deserialize, Serialize};
use xchain_crypto::sign::ecdsa::KeyPair;

/// 加载钱包地址或者加载enclave
#[derive(Default, Debug)]
pub struct Account {
    pub contract_name: String,
    pub contract_account: String,
    pub address: String,
    pub path: String,
}

impl Account {
    pub fn new(path: &str, contract_name: &str, contract_account: &str) -> Self {
        //加载私钥: features: normal | sgx | trustzone
        let p = xchain_crypto::account::get_ecdsa_private_key_from_file(path).unwrap();
        let alg = &xchain_crypto::sign::ecdsa::ECDSA_P256_SHA256_ASN1;
        let pk = xchain_crypto::account::PublicKey::new(alg, p.public_key());
        let address = xchain_crypto::account::address::get_address_from_public_key(&pk).unwrap();
        Account {
            address: address,
            path: path.to_string(),
            contract_account: contract_account.to_string(),
            contract_name: contract_name.to_string(),
        }
    }

    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let p = xchain_crypto::account::get_ecdsa_private_key_from_file(&self.path)?;
        //let alg = &xchain_crypto::sign::ecdsa::ECDSA_P256_SHA256_ASN1;
        Ok(p.sign(msg)?.as_ref().to_vec())
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<()> {
        let p = xchain_crypto::account::get_ecdsa_private_key_from_file(&self.path)?;
        let alg = &xchain_crypto::sign::ecdsa::ECDSA_P256_SHA256_ASN1;
        let pk = xchain_crypto::account::PublicKey::new(alg, p.public_key());
        pk.verify(msg, sig)?;
        Ok(())
    }

    pub fn public_key(&self) -> String {
        let p = xchain_crypto::account::get_ecdsa_private_key_from_file(&self.path).unwrap();
        xchain_crypto::account::json_key::get_ecdsa_public_key_json_format(&p).unwrap()
    }

    // TODO  把其他所有crypto相关的操作移动到这里
}

pub fn get_nonce() -> String {
    let t = super::consts::now_as_secs();
    let m: u32 = 100000000;

    let seed = xchain_crypto::hdwallet::rand::generate_seed_with_strength_and_keylen(
        xchain_crypto::hdwallet::rand::KeyStrength::HARD,
        64,
    )
    .unwrap();
    let mut same_seed = [0u8; 32];
    same_seed.copy_from_slice(&seed[..32]);
    let mut rng = StdRng::from_seed(same_seed);
    let r = rng.next_u32() % m;

    format!("{}{:08}", t, r)
}

#[allow(non_snake_case)]
#[derive(PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct TxInputDef {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ref_txid: ::std::vec::Vec<u8>,
    pub ref_offset: i32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub from_addr: ::std::vec::Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub amount: ::std::vec::Vec<u8>,
    #[serde(skip_serializing)]
    pub frozen_height: i64,
}

impl From<&xchain::TxInput> for TxInputDef {
    fn from(ti: &xchain::TxInput) -> Self {
        TxInputDef {
            ref_txid: ti.ref_txid.clone(),
            ref_offset: ti.ref_offset,
            from_addr: ti.from_addr.clone(),
            amount: ti.amount.clone(),
            frozen_height: ti.frozen_height,
        }
    }
}

#[allow(non_snake_case)]
#[derive(PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct TxInputExtDef {
    // message fields
    pub bucket: ::std::string::String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub key: ::std::vec::Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ref_txid: ::std::vec::Vec<u8>,
    pub ref_offset: i32,
}

impl From<&xchain::TxInputExt> for TxInputExtDef {
    fn from(tie: &xchain::TxInputExt) -> Self {
        TxInputExtDef {
            bucket: tie.bucket.to_owned(),
            key: tie.key.clone(),
            ref_txid: tie.ref_txid.clone(),
            ref_offset: tie.ref_offset,
        }
    }
}

#[allow(non_snake_case)]
#[derive(PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct TxOutputExtDef {
    // message fields
    pub bucket: ::std::string::String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub key: ::std::vec::Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub value: ::std::vec::Vec<u8>,
}

impl From<&xchain::TxOutputExt> for TxOutputExtDef {
    fn from(tie: &xchain::TxOutputExt) -> Self {
        TxOutputExtDef {
            bucket: tie.bucket.to_owned(),
            key: tie.key.clone(),
            value: tie.value.clone(),
        }
    }
}

#[allow(non_snake_case)]
#[derive(PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct TransactionDef {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tx_inputs: Vec<TxInputDef>,
    pub tx_outputs: Vec<xchain::TxOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub desc: ::std::vec::Vec<u8>,
    pub nonce: ::std::string::String,
    pub timestamp: i64,
    pub version: i32,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tx_inputs_ext: Vec<TxInputExtDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tx_outputs_ext: Vec<TxOutputExtDef>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contract_requests: Vec<xchain::InvokeRequest>,
    pub initiator: ::std::string::String,
    pub auth_require: Vec<std::string::String>,

    pub coinbase: bool,
    pub autogen: bool,

    //TODO 先不处理hd info
    #[serde(skip_serializing)]
    pub HD_info: Option<xchain::HDInfo>,
}

#[allow(non_snake_case)]
#[derive(PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct TransactionDefWithSigns {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tx_inputs: Vec<TxInputDef>,
    pub tx_outputs: Vec<xchain::TxOutput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub desc: ::std::vec::Vec<u8>,
    pub nonce: ::std::string::String,
    pub timestamp: i64,
    pub version: i32,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tx_inputs_ext: Vec<TxInputExtDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tx_outputs_ext: Vec<TxOutputExtDef>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contract_requests: Vec<xchain::InvokeRequest>,
    pub initiator: ::std::string::String,
    pub auth_require: Vec<::std::string::String>,

    //the only difference part
    pub initiator_signs: Vec<xchain::SignatureInfo>,
    pub auth_require_signs: Vec<xchain::SignatureInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xuper_sign: Option<xchain::XuperSignature>,

    pub coinbase: bool,
    pub autogen: bool,

    //TODO 先不处理hd info
    #[serde(skip_serializing)]
    pub HD_info: Option<xchain::HDInfo>,
}

impl From<&xchain::Transaction> for TransactionDefWithSigns {
    fn from(tx: &xchain::Transaction) -> Self {
        TransactionDefWithSigns {
            tx_inputs: tx
                .tx_inputs
                .clone()
                .into_vec()
                .iter()
                .map(|x| TxInputDef::from(x))
                .collect(),
            tx_outputs: tx.tx_outputs.clone().into_vec(),
            desc: tx.desc.clone(),
            nonce: tx.nonce.to_owned(),
            timestamp: tx.timestamp,
            version: tx.version,
            tx_inputs_ext: tx
                .tx_inputs_ext
                .clone()
                .into_vec()
                .iter()
                .map(|x| TxInputExtDef::from(x))
                .collect(),
            tx_outputs_ext: tx
                .tx_outputs_ext
                .clone()
                .into_vec()
                .iter()
                .map(|x| TxOutputExtDef::from(x))
                .collect(),
            contract_requests: tx.contract_requests.clone().into_vec(),
            initiator: tx.initiator.to_owned(),
            auth_require: tx.auth_require.clone().into_vec(),
            initiator_signs: tx.initiator_signs.clone().into_vec(),
            auth_require_signs: tx.auth_require_signs.clone().into_vec(),
            xuper_sign: tx.xuper_sign.clone().into_option(),
            coinbase: tx.coinbase,
            autogen: tx.autogen,
            HD_info: tx.HD_info.clone().into_option(),
        }
    }
}

impl From<&xchain::Transaction> for TransactionDef {
    fn from(tx: &xchain::Transaction) -> Self {
        TransactionDef {
            tx_inputs: tx
                .tx_inputs
                .clone()
                .into_vec()
                .iter()
                .map(|x| TxInputDef::from(x))
                .collect(),
            tx_outputs: tx.tx_outputs.clone().into_vec(),
            desc: tx.desc.clone(),
            nonce: tx.nonce.to_owned(),
            timestamp: tx.timestamp,
            version: tx.version,
            tx_inputs_ext: tx
                .tx_inputs_ext
                .clone()
                .into_vec()
                .iter()
                .map(|x| TxInputExtDef::from(x))
                .collect(),
            tx_outputs_ext: tx
                .tx_outputs_ext
                .clone()
                .into_vec()
                .iter()
                .map(|x| TxOutputExtDef::from(x))
                .collect(),
            contract_requests: tx.contract_requests.clone().into_vec(),
            initiator: tx.initiator.to_owned(),
            auth_require: tx.auth_require.clone().into_vec(),
            coinbase: tx.coinbase,
            autogen: tx.autogen,
            HD_info: tx.HD_info.clone().into_option(),
        }
    }
}

pub fn make_tx_digest_hash(tx: &xchain::Transaction) -> Result<Vec<u8>> {
    let d = TransactionDef::from(tx);
    let d = serde_json::to_string(&d)?;
    println!("make_transaction_id: {:?}", d);
    Ok(xchain_crypto::hash::hash::double_sha256(d.as_bytes()))
}

pub fn make_transaction_id(tx: &xchain::Transaction) -> Result<Vec<u8>> {
    let d = TransactionDefWithSigns::from(tx);
    let d = serde_json::to_string(&d)?;
    println!("make_transaction_id: {:?}", d);
    Ok(xchain_crypto::hash::hash::double_sha256(d.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_account() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("key/private.key");
        let acc = Account::new(d.to_str().unwrap(), "counter", "XC1111111111000000@xuper");
        println!("{:?}", acc);
        let address = include_str!("../key/address");
        assert_eq!(acc.address, address);
    }
}
