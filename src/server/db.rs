use std::sync::Arc;
use crate::types::{AddressStatus, HumanAddress};
use cdk_common::database::wallet::Database;

pub struct Db {
    db: Arc<dyn Database<cdk_common::database::Error> + Send + Sync>,
    primary_namespace: String,
}

const ADDRESS_NAMESPACE: &str = "paycodes";

impl Db {
    pub fn new(db: Arc<dyn Database<cdk_common::database::Error> + Send + Sync>, primary_namespace: String) -> Self {
        Self { db, primary_namespace }
    }

    pub async fn save_address(&self, address: &HumanAddress) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        let key = hex::encode(format!("{}@{}", address.user_name, address.domain));
        let bytes = serde_json::to_vec(address)?;
        self.db.kv_write(&self.primary_namespace, ADDRESS_NAMESPACE, &key, &bytes).await?;
        let duration = start.elapsed();
        tracing::info!(duration_ms = duration.as_millis(), "DB save_address completed");
        Ok(())
    }

    pub async fn find_active_address(
        &self,
        user_name: &str,
        domain: &str,
    ) -> Result<Option<HumanAddress>, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        let key = hex::encode(format!("{}@{}", user_name, domain));
        let value = self.db.kv_read(&self.primary_namespace, ADDRESS_NAMESPACE, &key).await?;
        let duration = start.elapsed();
        tracing::info!(duration_ms = duration.as_millis(), "DB find_active_address completed");

        match value {
            Some(v) => {
                let address: HumanAddress = serde_json::from_slice(&v)?;
                if address.status == AddressStatus::ACTIVE {
                    Ok(Some(address))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}
