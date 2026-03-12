use std::sync::Arc;
use crate::types::{PayCode, PayCodeStatus};
use cdk_common::database::wallet::Database;

pub struct Db {
    db: Arc<dyn Database<cdk_common::database::Error> + Send + Sync>,
    primary_namespace: String,
}

const SECONDARY_NAMESPACE: &str = "paycodes";

impl Db {
    pub fn new(db: Arc<dyn Database<cdk_common::database::Error> + Send + Sync>, primary_namespace: String) -> Self {
        Self { db, primary_namespace }
    }

    pub async fn save_paycode(&self, paycode: &PayCode) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        let key = hex::encode(format!("{}@{}", paycode.user_name, paycode.domain));
        let bytes = serde_json::to_vec(paycode)?;
        self.db.kv_write(&self.primary_namespace, SECONDARY_NAMESPACE, &key, &bytes).await?;
        let duration = start.elapsed();
        tracing::info!(duration_ms = duration.as_millis(), "DB save_paycode completed");
        Ok(())
    }

    pub async fn find_active_paycode(
        &self,
        user_name: &str,
        domain: &str,
    ) -> Result<Option<PayCode>, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        let key = hex::encode(format!("{}@{}", user_name, domain));
        let value = self.db.kv_read(&self.primary_namespace, SECONDARY_NAMESPACE, &key).await?;
        let duration = start.elapsed();
        tracing::info!(duration_ms = duration.as_millis(), "DB find_active_paycode completed");

        match value {
            Some(v) => {
                let paycode: PayCode = serde_json::from_slice(&v)?;
                if paycode.status == PayCodeStatus::ACTIVE {
                    Ok(Some(paycode))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}
