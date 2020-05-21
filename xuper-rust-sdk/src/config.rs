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
    #[serde(rename = "endorseServiceHost")]
    pub endorse_service_host: String,
    #[serde(rename = "complianceCheck")]
    pub compliance_check: ComplianceCheckConfig,
    #[serde(rename = "minNewChainAmount")]
    pub min_new_chain_amount: String,
    #[serde(rename = "crypto")]
    pub crypto: String,
}
