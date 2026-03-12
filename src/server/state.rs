use crate::server::cloudflare::CloudflareClient;
use crate::server::db::Db;
use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use std::collections::HashMap;
use std::sync::Arc;
use cdk::wallet::Wallet;
use cdk_sqlite::WalletSqliteDatabase;
use bip39::Mnemonic;
use cdk::nuts::CurrencyUnit;
use std::str::FromStr;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub cf: Arc<CloudflareClient>,
    pub custom_price_sats: u64,
    pub random_price_sats: u64,
    pub accepted_mints: Vec<String>,
    pub leptos_options: LeptosOptions,
    pub wallet: Arc<Wallet>,
    pub network: bitcoin::Network,
    pub app_name: String,
    pub default_domain: String,
    pub app_url: String,
}

impl AppState {
    pub async fn new(leptos_options: LeptosOptions) -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Nutty".to_string());
        let default_domain = std::env::var("DEFAULT_DOMAIN").unwrap_or_else(|_| "nutty.cash".to_string());
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| format!("https://{}", default_domain));

        let app_dir_name = format!(".{}", app_name.to_lowercase());
        let app_dir = home::home_dir()
            .ok_or("Could not find home directory")?
            .join(app_dir_name);
        
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)?;
        }

        let cf_token = std::env::var("CF_TOKEN").unwrap_or_default();
        let domains_json = std::env::var("DOMAINS").unwrap_or_else(|_| "{}".to_string());
        let domain_map: HashMap<String, String> =
            serde_json::from_str(&domains_json).unwrap_or_default();

        let network_str = std::env::var("NETWORK").unwrap_or_else(|_| "bitcoin".to_string());
        let network = bitcoin::Network::from_str(&network_str).unwrap_or(bitcoin::Network::Bitcoin);

        let cf = Arc::new(CloudflareClient::new(cf_token, domain_map, network_str, app_name.clone()));

        let custom_price_sats = std::env::var("CUSTOM_PRICE_SATS")
            .or_else(|_| std::env::var("PAYCODE_PRICE_SATS"))
            .unwrap_or_else(|_| "5000".to_string())
            .parse()
            .unwrap_or(5000);

        let random_price_sats = std::env::var("RANDOM_PRICE_SATS")
            .unwrap_or_else(|_| "0".to_string())
            .parse()
            .unwrap_or(0);

        let accepted_mints: Vec<String> = std::env::var("ACCEPTED_MINTS")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if accepted_mints.is_empty() {
            return Err("ACCEPTED_MINTS must be set".into());
        }

        let mnemonic_str = std::env::var("CDK_MNEMONIC").map_err(|_| "CDK_MNEMONIC not set")?;
        let mnemonic = Mnemonic::from_str(&mnemonic_str)?;
        let seed = mnemonic.to_seed("");

        let wallet_db_path = app_dir.join("wallet.db");
        let wallet_db =
            WalletSqliteDatabase::new(wallet_db_path.to_str().ok_or("Invalid path")?).await?;
        let wallet_db = Arc::new(wallet_db);

        let db = Arc::new(Db::new(wallet_db.clone(), app_name.to_lowercase()));

        let wallet = Wallet::new(
            &accepted_mints[0], // Primary mint URL
            CurrencyUnit::Sat,
            wallet_db,
            seed,
            None,
        )?;

        Ok(Self {
            db,
            cf,
            custom_price_sats,
            random_price_sats,
            accepted_mints,
            leptos_options,
            wallet: Arc::new(wallet),
            network,
            app_name,
            default_domain,
            app_url,
        })
    }
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}
