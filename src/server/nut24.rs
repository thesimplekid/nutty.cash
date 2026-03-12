use crate::bip21::create_bip21;
use crate::server::names::generate_random_name;
use crate::server::state::AppState;
use crate::types::{CreatePayCodeRequest, PayCode, PayCodeParam, PayCodeParamType, PayCodeStatus};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use cdk::mint_url::MintUrl;
use cdk::nuts::nut00::Token;
use cdk::nuts::nut18::PaymentRequest;
use cdk::nuts::CurrencyUnit;
use cdk::wallet::ReceiveOptions;
use cdk::Amount;
use chrono::Utc;
use std::str::FromStr;
use tracing::{error, instrument, warn};
use uuid::Uuid;
use serde_json::json;

#[instrument(skip(state, headers, payload))]
pub async fn handle_paycode_api(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreatePayCodeRequest>,
) -> Response {
    if let Err(e) = payload.validate() {
        warn!("Invalid request payload: {}", e);
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error": e}))).into_response();
    }

    let is_random = payload.user_name.is_none();
    let price_sats = if is_random {
        state.random_price_sats
    } else {
        state.custom_price_sats
    };

    // Check X-Cashu header for token
    let cashu_token_raw = headers.get("X-Cashu").and_then(|h| h.to_str().ok());
    let cashu_token = cashu_token_raw.and_then(|s| Token::from_str(s).ok());

    // If price > 0 and no valid token provided, we need payment
    if price_sats > 0 && cashu_token.is_none() {
        let user_name = if let Some(name) = payload.user_name.clone() {
            // Check availability for chosen name
            let (cf_exists, db_exists) = tokio::join!(
                state.cf.check_record_exists(&name, &payload.domain),
                state.db.find_active_paycode(&name, &payload.domain)
            );

            if cf_exists.unwrap_or(true) || db_exists.unwrap_or(None).is_some() {
                return (StatusCode::CONFLICT, Json(json!({"error": "Username taken"}))).into_response();
            }
            name
        } else {
            // Pick a random name for them
            let mut name = String::new();
            let mut success = false;
            for _ in 0..5 {
                name = generate_random_name();
                let (cf_exists, db_exists) = tokio::join!(
                    state.cf.check_record_exists(&name, &payload.domain),
                    state.db.find_active_paycode(&name, &payload.domain)
                );
                if !cf_exists.unwrap_or(true) && db_exists.unwrap_or(None).is_none() {
                    success = true;
                    break;
                }
            }
            if !success {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to generate unique name"}))).into_response();
            }
            name
        };

        // Build NUT-18 request
        let mut builder = PaymentRequest::builder()
            .payment_id(Uuid::new_v4().to_string())
            .amount(Amount::from(price_sats))
            .unit(CurrencyUnit::Sat)
            .description(format!("Purchase paycode: {}@{}", user_name, payload.domain));

        for mint in &state.accepted_mints {
            if let Ok(url) = MintUrl::from_str(mint) {
                builder = builder.add_mint(url);
            }
        }

        let req = builder.build();
        let encoded_req = req.to_string();

        return (
            StatusCode::PAYMENT_REQUIRED,
            [("X-Cashu", encoded_req)],
            Json(json!({
                "user_name": user_name,
                "amount": price_sats,
                "unit": "sat",
                "message": "Payment Required. See help link for instructions on how to purchase.",
                "accepted_mints": state.accepted_mints,
                "help": format!("{}/api/v1/SKILL.md", state.app_url)
            }))
        ).into_response();
    }

    // Validate token before doing anything else
    if let Some(ref token) = cashu_token {
        if token.unit() != Some(CurrencyUnit::Sat) {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Incorrect unit"}))).into_response();
        }

        let val = token.value().unwrap_or(Amount::ZERO);
        if val < Amount::from(price_sats) {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Insufficient amount"}))).into_response();
        }

        let mint_url = token.mint_url().map(|u| u.to_string()).unwrap_or_default();
        if !state.accepted_mints.contains(&mint_url) {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Mint not accepted"}))).into_response();
        }
    }

    // Verify username
    let user_name = match payload.user_name.clone() {
        Some(name) => name,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Username missing"}))).into_response();
        }
    };

    let (cf_exists, db_exists) = tokio::join!(
        state.cf.check_record_exists(&user_name, &payload.domain),
        state.db.find_active_paycode(&user_name, &payload.domain)
    );

    if cf_exists.unwrap_or(true) || db_exists.unwrap_or(None).is_some() {
        return (StatusCode::CONFLICT, Json(json!({"error": "Username taken (just now)"}))).into_response();
    }

    // Build paycode and BIP-21 URI
    let mut params = Vec::new();
    if let Some(v) = payload.lno { params.push(PayCodeParam { kind: PayCodeParamType::LNO, value: v, prefix: None }); }
    if let Some(v) = payload.sp { params.push(PayCodeParam { kind: PayCodeParamType::SP, value: v, prefix: None }); }
    if let Some(v) = payload.creq { params.push(PayCodeParam { kind: PayCodeParamType::CREQ, value: v, prefix: None }); }

    let paycode = PayCode {
        id: Uuid::new_v4(),
        created_at: Utc::now(),
        status: PayCodeStatus::ACTIVE,
        user_name,
        domain: payload.domain.clone(),
        params,
    };

    let bip21 = create_bip21(&paycode.params).unwrap_or_default();

    // Step 1: Create DNS record first (before taking payment)
    if let Err(e) = state.cf.create_txt_record(&paycode.user_name, &paycode.domain, &bip21).await {
        error!("Cloudflare record creation failed: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create DNS record"}))).into_response();
    }

    // Step 2: Redeem token (DNS record exists, so rollback on failure)
    if let Some(token) = cashu_token {
        let start = std::time::Instant::now();
        let res = state.wallet.receive(&token.to_string(), ReceiveOptions::default()).await;
        let duration = start.elapsed();
        tracing::info!(duration_ms = duration.as_millis(), "Cashu token redemption completed");

        if let Err(e) = res {
            error!("Failed to redeem token: {}, rolling back DNS record", e);
            if let Err(del_err) = state.cf.delete_txt_record(&paycode.user_name, &paycode.domain).await {
                error!("Failed to rollback DNS record: {}", del_err);
            }
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Payment redemption failed"}))).into_response();
        }

        if let Some(payout_creq) = state.payout_creq.as_deref() {
            match PaymentRequest::from_str(payout_creq) {
                Ok(payout_request) => {
                    match state.wallet.total_balance().await {
                        Ok(payout_amount) if payout_amount > Amount::ZERO => {
                            tracing::info!(amount = %payout_amount, "Attempting configured wallet balance payout");
                            if let Err(e) = state
                                .wallet
                                .pay_request(payout_request, Some(payout_amount))
                                .await
                            {
                                warn!(error = %e, amount = %payout_amount, "Configured payout request payment failed; ecash remains in wallet");
                            } else {
                                tracing::info!(amount = %payout_amount, "Configured wallet balance payout succeeded");
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            warn!(error = %e, "Failed to read wallet balance for configured payout request");
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "Configured payout request became invalid at runtime");
                }
            }
        }
    }

    // Step 3: Save to DB
    if let Err(e) = state.db.save_paycode(&paycode).await {
        error!("Failed to save ACTIVE paycode: {}", e);
        // DNS record exists and token is redeemed — log but don't rollback DNS
        // since the user has paid and the record is valid
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response();
    }

    (StatusCode::OK, Json(json!({
        "status": "active",
        "user_name": paycode.user_name,
        "domain": paycode.domain,
        "bip353": format!("{}@{}", paycode.user_name, paycode.domain)
    }))).into_response()
}
