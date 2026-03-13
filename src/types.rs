use crate::cashu::normalize_payment_request;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AddressStatus {
    PENDING,
    ACTIVE,
    EXPIRED,
    REVOKED,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AddressParamType {
    LNO,
    SP,
    CREQ,
    CUSTOM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAddressRequest {
    pub user_name: Option<String>,
    pub domain: String,
    pub lno: Option<String>,
    pub sp: Option<String>,
    pub creq: Option<String>,
}

impl CreateAddressRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref name) = self.user_name {
            if name.len() < 4 {
                return Err("Username must be at least 4 characters".into());
            }
            if !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
            {
                return Err("Accepted characters are: a-z, A-Z, 0-9, '.', '-', and '_'".into());
            }
        }

        if self.lno.is_none() && self.sp.is_none() && self.creq.is_none() {
            return Err("At least one payment option is required".into());
        }

        if let Some(ref lno) = self.lno {
            if !lno.starts_with("lno") {
                return Err("BOLT 12 offer must start with 'lno'".into());
            }
        }

        if let Some(ref sp) = self.sp {
            if !sp.starts_with("sp") {
                return Err("Silent payment address must start with 'sp'".into());
            }
        }

        if let Some(ref creq) = self.creq {
            normalize_payment_request(creq)
                .map(|_| ())
                .map_err(|_| "Cashu payment request must be a valid payment request".to_string())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressParam {
    pub prefix: Option<String>,
    pub value: String,
    pub kind: AddressParamType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanAddress {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub status: AddressStatus,
    pub user_name: String,
    pub domain: String,
    pub params: Vec<AddressParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LookupResult {
    pub address: String,
    pub uri: String,
    pub lno: Option<String>,
    pub sp: Option<String>,
    pub creq: Option<String>,
    pub onchain_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub app_name: String,
    pub default_domain: String,
}
