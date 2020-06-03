use crate::errors::*;
use crate::protos::xchain;
use rand::rngs::StdRng;
use rand_core::{RngCore, SeedableRng};
use serde::de::{MapAccess, Visitor};
use std::marker::PhantomData;

use serde::ser::SerializeSeq;
/// 保管私钥，提供签名和验签
/// 要在TEE里面运行
/// 唯一可以调用xchain_crypto的地方
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
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
        xchain_crypto::account::json_key::get_ecdsa_public_key_json_format_in_go(&p).unwrap()
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

fn encode_bytes(v: &Vec<u8>, buf: &mut String) -> Result<()> {
    if v.len() == 0 {
        return Ok(());
    }
    let b64 = base64::encode(v);
    let ti = serde_json::to_string(&b64)?;
    buf.push_str(&ti);
    buf.push('\n');
    Ok(())
}

pub fn serialize_bytes<S>(v: &Vec<u8>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    if v.len() == 0 {
        return serializer.serialize_none();
    }
    let b64 = base64::encode(v);
    serializer.serialize_str(&b64)
}

pub fn deserialize_bytes<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let s = String::deserialize(deserializer)?;
    let b64 = base64::decode(s).map_err(Error::custom)?;
    Ok(b64)
}

pub fn serialize_bytes_arr<S>(
    v: &protobuf::RepeatedField<Vec<u8>>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(v.len()))?;
    for e in v.iter() {
        let b64 = base64::encode(e);
        seq.serialize_element(&b64)?;
    }
    seq.end()
}

pub fn deserialize_bytes_arr<'de, D>(
    deserializer: D,
) -> std::result::Result<protobuf::RepeatedField<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let mut vec = Vec::new();
    let res = Vec::<u8>::deserialize(deserializer);
    for elem in res.iter() {
        let b64 = base64::decode(&elem).map_err(Error::custom)?;
        vec.push(b64);
    }
    Ok(protobuf::RepeatedField::from_vec(vec))
}

fn encode_array<T>(arr: &Vec<T>, buf: &mut String) -> Result<()>
where
    T: serde::ser::Serialize,
{
    if arr.len() > 0 {
        let s = serde_json::to_string(arr)?;
        buf.push_str(&s);
        buf.push('\n');
        return Ok(());
    }
    // push null
    buf.push('n');
    buf.push('u');
    buf.push('l');
    buf.push('l');
    buf.push('\n');
    Ok(())
}

pub fn is_zero(t: &i64) -> bool {
    t == &0
}

#[allow(non_snake_case)]
pub fn is_CPU(t: &crate::protos::xchain::ResourceType) -> bool {
    match t {
        crate::protos::xchain::ResourceType::CPU => true,
        _ => false,
    }
}

pub fn is_empty<T>(t: &protobuf::RepeatedField<T>) -> bool {
    t.is_empty()
}

pub fn serialize_ordered_map<S>(
    value: &HashMap<String, Vec<u8>>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    let mut res = BTreeMap::new();
    for (k, v) in ordered.iter() {
        res.insert(k, base64::encode(v));
    }
    res.serialize(serializer)
}

#[derive(Debug)]
struct MyMapVisitor<K, V> {
    marker: PhantomData<fn() -> HashMap<K, V>>,
}

impl<K, V> MyMapVisitor<K, V> {
    fn new() -> Self {
        MyMapVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, K, V> Visitor<'de> for MyMapVisitor<K, V>
where
    K: Deserialize<'de> + std::hash::Hash + std::cmp::Eq + std::fmt::Debug,
    V: Deserialize<'de> + std::fmt::Debug + AsRef<[u8]>,
{
    // The type that our Visitor is going to produce.
    type Value = HashMap<K, V>;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a very special map")
    }

    // Deserialize MyMap from an abstract "map" provided by the
    // Deserializer. The MapAccess input is a callback provided by
    // the Deserializer to let us see each entry in the map.
    fn visit_map<M>(self, mut access: M) -> std::result::Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }
        Ok(map)
    }
}

pub fn deserialize_ordered_map<'de, D>(
    deserializer: D,
) -> std::result::Result<HashMap<String, Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let res = deserializer.deserialize_map(MyMapVisitor::<String, String>::new())?;
    let mut ret = HashMap::new();
    for (k, v) in res.iter() {
        let b64 = base64::decode(&v).map_err(Error::custom)?;
        ret.insert(k.to_string(), b64);
    }
    Ok(ret)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxOutputDef {
    // message fields
    #[serde(serialize_with = "serialize_bytes")]
    pub amount: ::std::vec::Vec<u8>,
    #[serde(serialize_with = "serialize_bytes")]
    pub to_addr: ::std::vec::Vec<u8>,
    #[serde(skip_serializing_if = "is_zero")]
    pub frozen_height: i64,
}

impl From<&xchain::TxOutput> for TxOutputDef {
    fn from(to: &xchain::TxOutput) -> Self {
        TxOutputDef {
            amount: to.amount.clone(),
            to_addr: to.to_addr.clone(),
            frozen_height: to.frozen_height,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct SignatureInfoDef {
    // message fields
    pub PublicKey: ::std::string::String,
    #[serde(serialize_with = "serialize_bytes")]
    pub Sign: ::std::vec::Vec<u8>,
}

impl From<&xchain::SignatureInfo> for SignatureInfoDef {
    fn from(si: &xchain::SignatureInfo) -> Self {
        SignatureInfoDef {
            PublicKey: si.PublicKey.to_owned(),
            Sign: si.Sign.clone(),
        }
    }
}

#[allow(non_snake_case)]
pub struct TxOutputExtDef {
    // message fields
    pub bucket: ::std::string::String,
    pub key: ::std::vec::Vec<u8>,
    pub value: ::std::vec::Vec<u8>,
}

impl Serialize for TxOutputExtDef {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::Error;
        let mut j = String::new();

        let ti = serde_json::to_string(&self.bucket).map_err(Error::custom)?;
        j.push_str(&ti);
        j.push('\n');
        encode_bytes(&self.key, &mut j).unwrap();
        encode_bytes(&self.value, &mut j).unwrap();

        j.serialize(serializer)
    }
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
pub struct TransactionDef {
    pub tx_inputs: Vec<xchain::TxInput>,
    pub tx_outputs: Vec<TxOutputDef>,
    pub desc: ::std::vec::Vec<u8>,
    pub nonce: ::std::string::String,
    pub timestamp: i64,
    pub version: i32,

    pub tx_inputs_ext: Vec<xchain::TxInputExt>,
    pub tx_outputs_ext: Vec<xchain::TxOutputExt>,

    pub contract_requests: Vec<xchain::InvokeRequest>,
    pub initiator: ::std::string::String,
    pub auth_require: Vec<::std::string::String>,

    //the only difference part
    pub initiator_signs: Vec<SignatureInfoDef>,
    pub auth_require_signs: Vec<SignatureInfoDef>,
    pub xuper_sign: Option<xchain::XuperSignature>,

    pub coinbase: bool,
    pub autogen: bool,

    pub HD_info: Option<xchain::HDInfo>,
    pub include_signes: bool,
}

impl TransactionDef {
    fn serialize(&self) -> Result<String> {
        let mut j = String::new();
        for ti in &self.tx_inputs {
            encode_bytes(&ti.ref_txid, &mut j)?;
            let s = serde_json::to_string(&ti.ref_offset)?;
            j.push_str(&s);
            j.push('\n');
            encode_bytes(&ti.from_addr, &mut j)?;
            encode_bytes(&ti.amount, &mut j)?;
            let s = serde_json::to_string(&ti.frozen_height)?;
            j.push_str(&s);
            j.push('\n');
        }

        let s = serde_json::to_string(&self.tx_outputs)?;
        j.push_str(&s);
        j.push('\n');

        encode_bytes(&self.desc, &mut j)?;

        let s = serde_json::to_string(&self.nonce)?;
        j.push_str(&s);
        j.push('\n');

        let s = serde_json::to_string(&self.timestamp)?;
        j.push_str(&s);
        j.push('\n');

        let s = serde_json::to_string(&self.version)?;
        j.push_str(&s);
        j.push('\n');

        for tie in &self.tx_inputs_ext {
            let ti = serde_json::to_string(&tie.bucket)?;
            j.push_str(&ti);
            j.push('\n');

            encode_bytes(&tie.key, &mut j)?;
            encode_bytes(&tie.ref_txid, &mut j)?;

            let ti = serde_json::to_string(&tie.ref_offset)?;
            j.push_str(&ti);
            j.push('\n');
        }

        for toe in &self.tx_outputs_ext {
            let ti = serde_json::to_string(&toe.bucket)?;
            j.push_str(&ti);
            j.push('\n');
            encode_bytes(&toe.key, &mut j).unwrap();
            encode_bytes(&toe.value, &mut j).unwrap();
        }

        // map 按照key的字母顺序排列
        encode_array::<xchain::InvokeRequest>(&self.contract_requests, &mut j)?;

        let s = serde_json::to_string(&self.initiator)?;
        j.push_str(&s);
        j.push('\n');

        encode_array::<String>(&self.auth_require, &mut j)?;

        if self.include_signes {
            encode_array::<SignatureInfoDef>(&self.initiator_signs, &mut j)?;
            encode_array::<SignatureInfoDef>(&self.auth_require_signs, &mut j)?;

            if self.xuper_sign.is_some() {
                //TODO BUG
                let s = serde_json::to_string(&self.auth_require_signs)?;
                j.push_str(&s);
                j.push('\n');
            }
        }

        let s = serde_json::to_string(&self.coinbase)?;
        j.push_str(&s);
        j.push('\n');

        let s = serde_json::to_string(&self.autogen)?;
        j.push_str(&s);
        j.push('\n');

        if self.version > 2 {
            let s = serde_json::to_string(&self.HD_info)?;
            j.push_str(&s);
            j.push('\n');
        }

        Ok(j)
    }
}

impl From<&xchain::Transaction> for TransactionDef {
    fn from(tx: &xchain::Transaction) -> Self {
        TransactionDef {
            include_signes: false,
            tx_inputs: tx.tx_inputs.clone().into_vec(),
            tx_outputs: tx
                .tx_outputs
                .clone()
                .into_vec()
                .iter()
                .map(|x| TxOutputDef::from(x))
                .collect(),
            desc: tx.desc.clone(),
            nonce: tx.nonce.to_owned(),
            timestamp: tx.timestamp,
            version: tx.version,
            tx_inputs_ext: tx.tx_inputs_ext.clone().into_vec(),
            tx_outputs_ext: tx.tx_outputs_ext.clone().into_vec(),
            contract_requests: tx.contract_requests.clone().into_vec(),
            initiator: tx.initiator.to_owned(),
            auth_require: tx.auth_require.clone().into_vec(),
            initiator_signs: tx
                .initiator_signs
                .clone()
                .into_vec()
                .iter()
                .map(|x| SignatureInfoDef::from(x))
                .collect(),
            auth_require_signs: tx
                .auth_require_signs
                .clone()
                .into_vec()
                .iter()
                .map(|x| SignatureInfoDef::from(x))
                .collect(),
            xuper_sign: tx.xuper_sign.clone().into_option(),
            coinbase: tx.coinbase,
            autogen: tx.autogen,
            HD_info: tx.HD_info.clone().into_option(),
        }
    }
}

pub fn make_tx_digest_hash(tx: &xchain::Transaction) -> Result<Vec<u8>> {
    let d = TransactionDef::from(tx);
    let d = d.serialize()?;
    //notice: cryptos  do digest once default
    Ok(xchain_crypto::hash::hash::sha256(d.as_bytes()))
}

pub fn make_transaction_id(tx: &xchain::Transaction) -> Result<Vec<u8>> {
    let mut d = TransactionDef::from(tx);
    d.include_signes = true;
    let d = d.serialize()?;
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
