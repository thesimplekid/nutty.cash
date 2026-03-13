#[cfg(feature = "ssr")]
use bitcoin_payment_instructions::hrn_resolution::{HrnResolution, HrnResolver, HumanReadableName};
#[cfg(feature = "ssr")]
use bitcoin_payment_instructions::http_resolver::HTTPHrnResolver;
#[cfg(feature = "ssr")]
use bitcoin_payment_instructions::{
    PaymentInstructions, PaymentMethod, PossiblyResolvedPaymentMethod,
};

use crate::types::{AppConfig, CreatePayCodeRequest, LookupResult, PayCode};
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use std::str::FromStr;

#[server(name = GetAppConfig)]
pub async fn get_app_config() -> Result<AppConfig, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::server::state::AppState;
        let state = use_context::<AppState>().ok_or_else(|| ServerFnError::new("AppState not found"))?;

        Ok(AppConfig {
            app_name: state.app_name.clone(),
            default_domain: state.default_domain.clone(),
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(name = LookupBip353)]
pub async fn lookup_bip353(address: String) -> Result<LookupResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::server::state::AppState;
        let state = use_context::<AppState>().ok_or_else(|| ServerFnError::new("AppState not found"))?;

        let hrn = HumanReadableName::from_encoded(&address)
            .map_err(|e| ServerFnError::new(format!("Invalid address: {:?}", e)))?;

        let resolver = HTTPHrnResolver::default();
        let resolution = resolver
            .resolve_hrn(&hrn)
            .await
            .map_err(|e| ServerFnError::new(format!("BIP-353 resolution failed: {:?}", e)))?;

        let uri = match resolution {
            HrnResolution::DNSSEC { result, .. } => result,
            HrnResolution::LNURLPay { .. } => {
                return Err(ServerFnError::new("LNURL-pay not supported via BIP-353"))
            }
        };

        let instructions = PaymentInstructions::parse(&uri, state.network, &resolver, true)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to parse BIP-321 URI: {:?}", e)))?;

        let mut res = LookupResult {
            address: address.clone(),
            uri: uri.clone(),
            lno: None,
            sp: None,
            creq: None,
            onchain_address: None,
        };

        match instructions {
            PaymentInstructions::FixedAmount(f) => {
                for method in f.methods() {
                    match method {
                        PaymentMethod::LightningBolt12(offer) => {
                            res.lno = Some(offer.to_string());
                        }
                        PaymentMethod::OnChain(addr) => {
                            res.onchain_address = Some(addr.to_string());
                        }
                        PaymentMethod::Cashu(req) => {
                            res.creq = Some(req.to_string());
                        }
                        _ => {}
                    }
                }
            }
            PaymentInstructions::ConfigurableAmount(c) => {
                for method in c.methods() {
                    match method {
                        PossiblyResolvedPaymentMethod::Resolved(resolved) => match resolved {
                            PaymentMethod::LightningBolt12(offer) => {
                                res.lno = Some(offer.to_string());
                            }
                            PaymentMethod::OnChain(addr) => {
                                res.onchain_address = Some(addr.to_string());
                            }
                            PaymentMethod::Cashu(req) => {
                                res.creq = Some(req.to_string());
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }

        Ok(res)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = address;
        unreachable!()
    }
}

#[server(name = CreatePaycodeServer)]
pub async fn create_paycode_server(req: CreatePayCodeRequest) -> Result<PayCode, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::bip21::create_bip21;
        use crate::cashu::normalize_payment_request;
        use crate::server::state::AppState;
        use crate::types::{PayCodeParam, PayCodeParamType, PayCodeStatus};
        use axum::http::HeaderMap;
        use cdk::nuts::nut18::PaymentRequest;
        use chrono::Utc;
        use tracing::{error, warn};
        use uuid::Uuid;

        let state = use_context::<AppState>().ok_or_else(|| {
            error!("AppState not found in context during create_paycode_server");
            ServerFnError::new("AppState not found in context")
        })?;

        let headers: HeaderMap = leptos_axum::extract().await.map_err(|e| {
            error!("Failed to extract HeaderMap: {:?}", e);
            ServerFnError::new("Internal error")
        })?;

        if let Err(e) = req.validate() {
            warn!(error = %e, "Invalid paycode request validation failed");
            return Err(ServerFnError::new(e));
        }

        let is_random = req.user_name.is_none();
        let price_sats = if is_random {
            state.random_price_sats
        } else {
            state.custom_price_sats
        };

        // Check X-Cashu header for token
        let cashu_token_str = headers.get("X-Cashu").and_then(|h| h.to_str().ok());

        // If payment is required and no token is provided, return an error that the frontend can handle
        if price_sats > 0 && cashu_token_str.is_none() {
            let user_name = if is_random {
                let mut name = String::new();
                let mut success = false;
                for _ in 0..5 {
                    name = crate::server::names::generate_random_name();
                    let (cf_exists, db_exists) = tokio::join!(
                        state.cf.check_record_exists(&name, &req.domain),
                        state.db.find_active_paycode(&name, &req.domain)
                    );
                    if !cf_exists.unwrap_or(true) && db_exists.unwrap_or(None).is_none() {
                        success = true;
                        break;
                    }
                }
                if !success {
                    return Err(ServerFnError::new("Failed to generate unique name"));
                }
                name
            } else {
                req.user_name.clone().unwrap()
            };

            // Check availability for chosen name
            if !is_random {
                let (cf_exists, db_exists) = tokio::join!(
                    state.cf.check_record_exists(&user_name, &req.domain),
                    state.db.find_active_paycode(&user_name, &req.domain)
                );
                if cf_exists.unwrap_or(true) || db_exists.unwrap_or(None).is_some() {
                    return Err(ServerFnError::new("Username taken"));
                }
            }

            // Return a structured error for the frontend (JSON payload after prefix)
            let payment_data = serde_json::json!({
                "amount": price_sats,
                "user_name": user_name,
                "accepted_mints": state.accepted_mints,
                "message": "Payment Required. See help link for instructions on how to purchase.",
                "help": format!("{}/api/v1/SKILL.md", state.app_url)
            });
            return Err(ServerFnError::new(format!(
                "PAYMENT_REQUIRED:{}",
                payment_data
            )));
        }

        let user_name = if let Some(ref name) = req.user_name {
            name.clone()
        } else {
            // This case should be handled by the frontend providing the name back once it has it.
            return Err(ServerFnError::new("Username missing"));
        };

        // Validate token upfront (before any side effects)
        let validated_token = if price_sats > 0 {
            let token_str = cashu_token_str.unwrap();
            use cdk::nuts::nut00::Token;
            use cdk::nuts::CurrencyUnit;
            use cdk::Amount;
            use std::str::FromStr;

            let token = Token::from_str(token_str)
                .map_err(|e| ServerFnError::new(format!("Invalid token: {}", e)))?;

            if token.unit() != Some(CurrencyUnit::Sat) {
                return Err(ServerFnError::new("Incorrect currency unit"));
            }

            let token_value = token
                .value()
                .map_err(|e| ServerFnError::new(format!("Invalid token value: {}", e)))?;
            if token_value < Amount::from(price_sats) {
                return Err(ServerFnError::new("Insufficient amount"));
            }

            let mint_url = token
                .mint_url()
                .map_err(|e| ServerFnError::new(format!("Invalid mint URL: {}", e)))?
                .to_string();
            if !state.accepted_mints.contains(&mint_url) {
                return Err(ServerFnError::new("Mint not accepted"));
            }

            // Re-check availability
            let (cf_exists, db_exists) = tokio::join!(
                state.cf.check_record_exists(&user_name, &req.domain),
                state.db.find_active_paycode(&user_name, &req.domain)
            );

            if cf_exists.unwrap_or(true) || db_exists.unwrap_or(None).is_some() {
                return Err(ServerFnError::new("Username taken (just now)"));
            }

            Some(token)
        } else {
            None
        };

        let payout_request = state
            .payout_creq
            .as_deref()
            .map(PaymentRequest::from_str)
            .transpose()
            .map_err(|e| ServerFnError::new(format!("Invalid payout request: {}", e)))?;

        let mut params = Vec::new();
        if let Some(v) = req.lno {
            params.push(PayCodeParam {
                kind: PayCodeParamType::LNO,
                value: v,
                prefix: None,
            });
        }
        if let Some(v) = req.sp {
            params.push(PayCodeParam {
                kind: PayCodeParamType::SP,
                value: v,
                prefix: None,
            });
        }
        if let Some(v) = req.creq {
            let normalized_creq = normalize_payment_request(&v)
                .map_err(ServerFnError::new)?;
            params.push(PayCodeParam {
                kind: PayCodeParamType::CREQ,
                value: normalized_creq,
                prefix: None,
            });
        }

        let paycode = PayCode {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            status: PayCodeStatus::ACTIVE,
            user_name,
            domain: req.domain,
            params,
        };

        let bip21 = create_bip21(&paycode.params).map_err(|e| {
            error!(error = %e, "Failed to build BIP-21 URI");
            ServerFnError::new(e)
        })?;

        // Step 1: Create DNS record first (before taking payment)
        state
            .cf
            .create_txt_record(&paycode.user_name, &paycode.domain, &bip21)
            .await
            .map_err(|e| {
                error!(error = %e, "Cloudflare API record creation failed");
                ServerFnError::new("Failed to create DNS record")
            })?;

        // Step 2: Redeem token (DNS exists, rollback on failure)
        if let Some(token) = validated_token {
            use cdk::wallet::ReceiveOptions;

            let mint_url = token
                .mint_url()
                .map_err(|e| ServerFnError::new(format!("Invalid token mint URL: {}", e)))?;

            let wallet = state
                .sat_wallet_for_mint(&mint_url)
                .await
                .map_err(|e| {
                    error!(mint = %mint_url, error = %e, "Failed to load wallet for token mint");
                    ServerFnError::new("Mint wallet unavailable")
                })?;

            let start = std::time::Instant::now();
            let res = wallet.receive(&token.to_string(), ReceiveOptions::default()).await;
            let duration = start.elapsed();
            tracing::info!(mint = %mint_url, duration_ms = duration.as_millis(), "Cashu token redemption completed");

            if let Err(e) = res {
                error!(error = %e, "Failed to redeem token, rolling back DNS record");
                if let Err(del_err) = state
                    .cf
                    .delete_txt_record(&paycode.user_name, &paycode.domain)
                    .await
                {
                    error!(error = %del_err, "Failed to rollback DNS record");
                }
                return Err(ServerFnError::new("Payment redemption failed"));
            }

            if let Some(ref payout_request) = payout_request {
                match wallet.total_balance().await {
                    Ok(payout_amount) if payout_amount > cdk::Amount::ZERO => {
                        tracing::info!(mint = %mint_url, amount = %payout_amount, "Attempting configured wallet balance payout");
                        if let Err(e) = wallet
                            .pay_request(payout_request.clone(), Some(payout_amount))
                            .await
                        {
                            warn!(mint = %mint_url, error = %e, amount = %payout_amount, "Configured payout request payment failed; ecash remains in wallet");
                        } else {
                            tracing::info!(mint = %mint_url, amount = %payout_amount, "Configured wallet balance payout succeeded");
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!(mint = %mint_url, error = %e, "Failed to read wallet balance for configured payout request");
                    }
                }
            }
        }

        // Step 3: Save to DB
        state.db.save_paycode(&paycode).await.map_err(|e| {
            error!(error = %e, "Failed to save paycode to database");
            ServerFnError::new(e.to_string())
        })?;

        Ok(paycode)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}
