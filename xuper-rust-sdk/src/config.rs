use serde::{Deserialize, Serialize};

//TODO: handle skip
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ComplianceCheckConfig {
    #[serde(rename = "isNeedComplianceCheck", skip)]
    pub is_need_compliance_check: bool,
    #[serde(rename = "isNeedComplianceCheckFee", skip)]
    pub is_need_compliance_fee: bool,
    #[serde(rename = "complianceCheckEndorseServiceFee")]
    pub compliance_check_endorse_service_fee: i32,
    #[serde(rename = "complianceCheckEndorseServiceFeeAddr")]
    pub compliance_check_endorse_service_fee_addr: String,
    #[serde(rename = "complianceCheckEndorseServiceAddr")]
    pub compliance_check_endorse_service_addr: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CommConfig {
    #[serde(rename = "node")]
    pub node: String,
    #[serde(rename = "nodePort")]
    pub node_port: u16,
    #[serde(rename = "endorsePort")]
    pub endorse_port: u16,
    #[serde(rename = "complianceCheck")]
    pub compliance_check: ComplianceCheckConfig,
    #[serde(rename = "minNewChainAmount", skip)]
    pub min_new_chain_amount: String,
    #[serde(rename = "crypto")]
    pub crypto: String,
}

lazy_static! {
    pub static ref CONFIG: std::sync::RwLock<CommConfig> = {
        let contents = include_str!("../conf/sdk.yaml");
        let yaml: CommConfig = serde_yaml::from_str(&contents).expect("serde_yaml");
        std::sync::RwLock::new(yaml)
    };
}
