#[cfg(feature = "ssr")]
use cdk::nuts::nut18::PaymentRequest;
#[cfg(feature = "ssr")]
use std::str::FromStr;

#[cfg(feature = "ssr")]
pub fn normalize_payment_request(creq: &str) -> Result<String, String> {
    PaymentRequest::from_str(creq)
        .map_err(|e| e.to_string())
        .and_then(|request| request.to_bech32_string().map_err(|e| e.to_string()))
}

#[cfg(not(feature = "ssr"))]
pub fn normalize_payment_request(_creq: &str) -> Result<String, String> {
    Ok(String::new())
}
