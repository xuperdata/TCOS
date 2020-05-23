use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ComplianceCheckConfig {
    #[serde(rename = "isNeedComplianceCheck")]
    pub is_need_compliance_check: bool,
    #[serde(rename = "isNeedComplianceCheckFee")]
    pub is_need_compliance_fee: bool,
    #[serde(rename = "complianceCheckEndorseServiceFee")]
    pub compliance_check_endorse_service_fee: i32,
    #[serde(rename = "complianceCheckEndorseServiceFeeAddr")]
    pub compliance_check_endorse_service_fee_addr: String,
    #[serde(rename = "complianceCheckEndorseServiceAddr")]
    pub compliance_check_endorse_service_addr: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CommConfig {
    #[serde(rename = "node")]
    pub node_port: String,
    #[serde(rename = "nodePort")]
    pub node_port: u16,
    #[serde(rename = "endorsePort")]
    pub endorse_port: u16,
    #[serde(rename = "complianceCheck")]
    pub compliance_check: ComplianceCheckConfig,
    #[serde(rename = "minNewChainAmount")]
    pub min_new_chain_amount: String,
    #[serde(rename = "crypto")]
    pub crypto: String,
}

lazy_static! {
    pub static ref CONFIG: RwLock<CommConfig> = {
        let path = std::path::PathBuf(std::env::var("CONFIG").unwrap());
        let f = std::fs::File::open(path).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("read_to_string");
        let yaml: CommConfig = serde_yaml::from_str(&contents).expect("serde_yaml");
        RwLock::new(yaml)
    };
}
