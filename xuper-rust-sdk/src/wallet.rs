use crate::errors::*;
use crate::protos::xchain;
/// 保管私钥，提供签名和验签
/// 要在TEE里面运行
/// 唯一可以调用xchain_crypto的地方
use serde::ser::{Serialize, SerializeSeq, Serializer};
use rand_core::{RngCore, SeedableRng};
use rand::rngs::StdRng;
use xchain_crypto::sign::ecdsa::KeyPair;

/// 加载钱包地址或者加载enclave
#[derive(Default)]
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
        Account{
            address: address,
            path: path.to_string(),
            contract_account: contract_account.to_string(),
            contract_name: contract_name.to_string(),
        }
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let p = xchain_crypto::account::get_ecdsa_private_key_from_file(&self.path).unwrap();
        let alg = &xchain_crypto::sign::ecdsa::ECDSA_P256_SHA256_ASN1;
        p.sign(msg).unwrap().as_ref().to_vec()
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8])-> bool {
        let p = xchain_crypto::account::get_ecdsa_private_key_from_file(&self.path).unwrap();
        let alg = &xchain_crypto::sign::ecdsa::ECDSA_P256_SHA256_ASN1;
        let pk = xchain_crypto::account::PublicKey::new(alg, p.public_key());
        pk.verify(msg, sig).is_ok()
    }

    pub fn public_key(&self) -> Vec<u8> {
        let p = xchain_crypto::account::get_ecdsa_private_key_from_file(&self.path).unwrap();
        let alg = &xchain_crypto::sign::ecdsa::ECDSA_P256_SHA256_ASN1;
        let pk = xchain_crypto::account::PublicKey::new(alg, p.public_key());
        pk.as_ref().to_vec()
    }

    // TODO  把其他所有crypto相关的操作移动到这里
}

/*
pub fn set_seed() -> Result<MyRng> {
    let seed = xchain_crypto::hdwallet::rand::generate_seed_with_strength_and_keylen(
        xchain_crypto::hdwallet::rand::KeyStrength::HARD,
        64,
    )?;
    let same_seed = MyRngSeed(&seed[..]);
    Ok(SeedableRng::from_seed(same_seed))
}
*/

pub fn get_nonce() -> String {
    let t = super::consts::now_as_secs();
    let m: u32 = 100000000;

    let seed = xchain_crypto::hdwallet::rand::generate_seed_with_strength_and_keylen(
        xchain_crypto::hdwallet::rand::KeyStrength::HARD,
        64,
    ).unwrap();
    let mut same_seed = [0u8; 32];
    let bytes = &seed[..same_seed.len()]; // panics if not enough data
    same_seed.copy_from_slice(&seed);
    //let same_seed = MyRngSeed(same_seed);
    //let mut pseudo_rng = MyRng::from_seed(same_seed);

    //let r = pseudo_rng.gen_range(1, m);
    let mut rng = StdRng::from_seed(same_seed);
    let r = rng.next_u32() % m;

    format!("{}{:08}", t, r)
}

pub struct TransactionWrapper<'a> {
    tx: &'a xchain::Transaction,
    include_signs: bool,
}

impl<'a> Serialize for TransactionWrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        for ti in self.tx.tx_inputs.iter() {
            if ti.ref_txid.len() > 0 {
                serializer.serialize_bytes(&ti.ref_txid)?;
            }
            serializer.serialize_i32(ti.ref_offset)?;
            if ti.from_addr.len() > 0 {
                serializer.serialize_bytes(&ti.from_addr)?;
            }
            serializer.serialize_bytes(&ti.amount)?;
            serializer.serialize_i64(ti.frozen_height)?;
        }
        if self.tx.desc.len() > 0 {
            serializer.serialize_bytes(&self.tx.desc)?;
        }
        serializer.serialize_bytes(&self.tx.nonce.as_bytes())?;
        serializer.serialize_i64(self.tx.timestamp)?;
        serializer.serialize_i32(self.tx.version)?;

        // 读写集
        for tie in self.tx.tx_inputs_ext.iter() {
            serializer.serialize_bytes(&tie.bucket.as_bytes())?;
            if tie.key.len() > 0 {
                serializer.serialize_bytes(&tie.key)?;
            }
            if tie.ref_txid.len() > 0 {
                serializer.serialize_bytes(&tie.ref_txid)?;
            }
            serializer.serialize_i32(tie.ref_offset)?;
        }
        for tie in self.tx.tx_outputs_ext.iter() {
            serializer.serialize_bytes(&tie.bucket.as_bytes())?;
            if tie.key.len() > 0 {
                serializer.serialize_bytes(&tie.key)?;
            }
            if tie.value.len() > 0 {
                serializer.serialize_bytes(&tie.value)?;
            }
        }

        let mut seq = serializer.serialize_seq(Some(self.tx.contract_requests.len()))?;
        for elem in self.tx.contract_requests.iter() {
//            seq.serialize_element(elem)?;
        }
        seq.end();
        serializer.serialize_bytes(&self.tx.initiator.as_bytes())?;

        let mut seq = serializer.serialize_seq(Some(self.tx.auth_require.len()))?;
        for elem in self.tx.auth_require.iter() {
 //           seq.serialize_element(elem)?;
        }
        seq.end();

        /*
        if self.include_signs {
            let mut seq = serializer.serialize_seq(Some(self.tx.initiator_signs.len()))?;
            for elem in self.tx.initiator_signs {
                seq.serialize_element(elem)?;
            }
            seq.end();
            let mut seq = serializer.serialize_seq(Some(self.tx.auth_require_signs.len()))?;
            for elem in self.tx.auth_require_signs {
                seq.serialize_element(elem)?;
            }
            if self.tx.xuper_sign.is_some() {
                seq.serialize_element(&self.tx.xuper_sign);
            }
            seq.end();
        }
        */

        serializer.serialize_bool(self.tx.coinbase)?;
        serializer.serialize_bool(self.tx.autogen)?;
        /*
        if self.tx.version > 2 {
            let mut hd_info = serializer.serialize_struct("HD_Info", 2)?;
            hd_info.serialize_field("hd_public_key", &self.tx.HD_info.hd_public_key)?;
            hd_info.serialize_field("original_hash", &self.tx.HD_info.original_hash)?;
            hd_info.end();
        }*/
        serializer.serialize_unit()
    }
}

pub fn make_tx_digest_hash<'a>(tx: &'a xchain::Transaction) -> Result<Vec<u8>> {
    let d = TransactionWrapper::<'a> {
        tx: tx,
        include_signs: false,
    };
    let d = serde_json::to_string(&d)?;
    Ok(xchain_crypto::hash::hash::double_sha256(d.as_bytes()))
}

pub fn make_transaction_id<'a>(tx: &'a xchain::Transaction) -> Result<Vec<u8>> {
    let d = TransactionWrapper::<'a> {
        tx: tx,
        include_signs: true,
    };
    let d = serde_json::to_string(&d)?;
    Ok(xchain_crypto::hash::hash::double_sha256(d.as_bytes()))
}
