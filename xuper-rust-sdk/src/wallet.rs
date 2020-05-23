/// 保管私钥，提供签名和验签
/// 要在TEE里面运行
/// 唯一可以调用xchain_crypto的地方
use serde::ser::{Serialize, SerializeSeq, Serializer};
use crate::errors::*;

use crate::protos::xchain;

use rand_core::SeedableRng;

/// 加载钱包地址或者加载enclave
pub struct Account {
    contract_name: String,
    contract_account: String,
    address: String,
}

impl Account {
    pub fn new(path: std::path::PathBuf) -> Self {
        //加载私钥: features: normal | sgx | trustzone
    }

    pub fn sign() {

    }

    pub fn verify() {

    }

    // TODO  把其他所有crypto相关的操作移动到这里
}


pub fn set_seed() -> Result<SeedableRng> {
    let seed = xchain_crypto::hdwallet::rand::generate_seed_with_strength_and_keylen(
       xchain_crypto::hdwallet::rand::KeyStrength::HARD,
        64,
    )?;
    let same_seed = MyRngSeed(&seed);
    Ok(SeedableRng::from_seed(same_seed))
}

pub fn get_nonce() -> String {
    let t = super::consts::now_as_secs();
    let m: u32 = 100000000;
    let r =  set_seed()?.next_u32() % m;
    std::fmt::format!("{}{:08}", t, r)
}


const N: usize = 64;
pub struct MyRngSeed(pub [u8; N]);
pub struct MyRng(MyRngSeed);

impl Default for MyRngSeed {
    fn default() -> MyRngSeed {
        MyRngSeed([0; N])
    }
}

impl AsMut<[u8]> for MyRngSeed {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl SeedableRng for MyRng {
    type Seed = MyRngSeed;

    fn from_seed(seed: MyRngSeed) -> MyRng {
        MyRng(seed)
    }
}

#[test]
fn test_myrng () {
    let same_seed = MyRngSeed( [42 as u8; 64] );
    let mut pseudo_rng = SeedableRng::from_seed( same_seed );
    let rnd_nmr= pseudo_rng.gen_range(0, 90);
    println!("{}", rnd_nmr);
}


pub struct TransactionWrapper {
    tx: xchain:Transaction,
    include_signs: bool,
}

impl Serialize for TransactionWrapper {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        for ti in self.tx.tx_inputs {
            if ti.ref_txid.len() > 0 {
                serializer.serialize_bytes(&ti.ref_txid)?;
            }
            serializer.serialize_i32(ti.ref_offset())?;
            if tx_inputs.from_addr.len() > 0 {
                serializer.serialize_bytes(&ti.from_addr)?;
            }
            serializer.serialize_bytes(&ti.amount)?;
            serializer.serialize_i64(ti.frozen_height)?;
        }
        if self.tx.desc.len() > 0 {
            serializer.serialize_bytes(&self.tx.desc)?;
        }
        serializer.serialize_bytes(&self.tx.nonce.as_bytes())?;
        serializer.serialize_i64(ti.timestamp)?;
        serializer.serialize_i32(ti.version)?;

        // 读写集
        for tie in self.tx.tx_inputs_ext {
            serializer.serialize_bytes(&tie.bucket.as_bytes())?;
            if tie.key.len() > 0 {
                serialier.serialize_bytes(&tie.key)?;
            }
            if tie.ref_txid.len() > 0 {
                serialier.serialize_bytes(&tie.ref_txid)?;
            }
            serialier.serialize_i32(tie.ref_offset)?;
        }
        for tie in self.tx.tx_outputs_ext {
            serializer.serialize_bytes(&tie.bucket.as_bytes())?;
            if tie.key.len() > 0 {
                serialier.serialize_bytes(&tie.key)?;
            }
            if tie.value.len() > 0 {
                serialier.serialize_bytes(&tie.value)?;
            }
        }

        let mut seq = serializer.serialize_seq(Some(self.tx.contract_requests.len()))?;
        for elem in self.tx.contract_requests {
            seq.serialize_elem(elem)?;
        }
        seq.end();
        serialier.serialize_bytes(&self.tx.initiator.as_bytes())?;

        let mut seq = serializer.serialize_seq(Some(self.tx.auth_require.len()))?;
        for elem in self.tx.auth_require {
            seq.serialize_elem(elem)?;
        }
        seq.end();

        if self.include_signs {
            let mut seq = serializer.serialize_seq(Some(self.tx.initiator_signs.len()))?;
            for elem in self.tx.initiator_signs {
                req.serialize_elem(elem)?;
            }
            let mut seq = serializer.serialize_seq(Some(self.tx.auth_require_signs.len()))?;
            for elem in self.tx.auth_require_signs {
                req.serialize_elem(elem)?;
            }
            if self.tx.xuper_sign.is_some() {
                req.serialize_elem(self.tx.xuper_sign);
            }
        }

        serialier.serialize_bool(self.tx.coinbase)?;
        serialier.serialize_bool(self.tx.autogen)?;
        if self.tx.version > 2 {
            let mut hd_info = serializer.serialize_struct("HD_Info", 2)?;
            hd_info.serialize_field("hd_public_key", &self.tx.HD_info.hd_public_key)?;
            hd_info.serialize_field("original_hash", &self.tx.HD_info.original_hash)?;
            hd_info.end()
        }
    }
}

pub fn make_tx_digest_hash(tx: &xchain::Transaction) -> Result<Vec<u8>> {
    let d = TransactionWrapper{
        tx: tx,
        include_signs: false,
    };
    let d = serde_json::to_string(d)?;
    xchain_crypto::hash::double_sha256(d)
}

pub fn make_transaction_id(tx: &xchain::Transaction) -> Result<Vec<u8>> {
    let d = TransactionWrapper{
        tx: tx,
        include_signs: true,
    };
    let d = serde_json::to_string(d)?;
    xchain_crypto::hash::double_sha256(d)
}

