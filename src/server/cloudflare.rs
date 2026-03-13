use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, error, instrument};

pub struct CloudflareClient {
    token: String,
    domain_map: HashMap<String, String>,
    network: String,
    client: reqwest::Client,
    app_name: String,
}

impl CloudflareClient {
    pub fn new(token: String, domain_map: HashMap<String, String>, network: String, app_name: String) -> Self {
        let client = reqwest::Client::new();
        Self { token, domain_map, network, client, app_name }
    }

    fn get_full_record_name(&self, user_name: &str, domain: &str) -> String {
        if self.network.is_empty() || self.network == "bitcoin" {
            format!("{}.user._bitcoin-payment.{}", user_name, domain)
        } else {
            format!("{}.user._bitcoin-payment.{}.{}", user_name, self.network, domain)
        }
    }

    #[instrument(skip(self), fields(user_name = %user_name, domain = %domain))]
    pub async fn check_record_exists(&self, user_name: &str, domain: &str) -> Result<bool, String> {
        let zone_id = self.domain_map.get(domain).ok_or_else(|| {
            let err = format!("Domain {} not configured", domain);
            error!("{}", err);
            err
        })?;
        
        let full_name = self.get_full_record_name(user_name, domain);

        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}&type=TXT", zone_id, full_name);
        
        let start = std::time::Instant::now();
        let res = self.client.get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        let duration = start.elapsed();
        info!(duration_ms = duration.as_millis(), "Cloudflare check_record_exists API request completed");

        let res = res.map_err(|e| {
                let err = e.to_string();
                error!("Cloudflare request failed: {}", err);
                err
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Cloudflare API error ({}): {}", status, error_text);
            return Err(format!("Cloudflare API error ({}): {}", status, error_text));
        }

        let data: Value = res.json().await.map_err(|e| {
            let err = e.to_string();
            error!("Failed to parse Cloudflare response: {}", err);
            err
        })?;
        let result = data["result"].as_array().ok_or_else(|| {
            error!("Invalid response from Cloudflare: 'result' field missing or not an array");
            "Invalid response from Cloudflare"
        })?;
        
        let exists = !result.is_empty();
        info!(exists = %exists, "Checked record existence");
        Ok(exists)
    }

    #[instrument(skip(self, content), fields(user_name = %user_name, domain = %domain))]
    pub async fn create_txt_record(&self, user_name: &str, domain: &str, content: &str) -> Result<(), String> {
        let zone_id = self.domain_map.get(domain).ok_or_else(|| {
            let err = format!("Domain {} not configured", domain);
            error!("{}", err);
            err
        })?;
        
        let full_name = self.get_full_record_name(user_name, domain);

        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zone_id);
        
        let body = json!({
            "content": format!("\"{}\"", content),
            "name": full_name,
            "proxied": false,
            "type": "TXT",
            "comment": format!("{} User DNS Update", self.app_name),
            "ttl": 3600
        });

        let start = std::time::Instant::now();
        let res = self.client.post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await;
        let duration = start.elapsed();
        info!(duration_ms = duration.as_millis(), "Cloudflare create_txt_record API request completed");

        let res = res.map_err(|e| {
                let err = e.to_string();
                error!("Cloudflare request failed: {}", err);
                err
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Cloudflare API error ({}): {}", status, error_text);
            return Err(format!("Cloudflare API error ({}): {}", status, error_text));
        }

        info!("Successfully created TXT record");
        Ok(())
    }

    #[instrument(skip(self), fields(user_name = %user_name, domain = %domain))]
    pub async fn delete_txt_record(&self, user_name: &str, domain: &str) -> Result<(), String> {
        let zone_id = self.domain_map.get(domain).ok_or_else(|| {
            let err = format!("Domain {} not configured", domain);
            error!("{}", err);
            err
        })?;

        let full_name = self.get_full_record_name(user_name, domain);

        // First, find the record ID
        let list_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}&type=TXT",
            zone_id, full_name
        );

        let start_list = std::time::Instant::now();
        let res = self.client.get(&list_url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await;
        let duration_list = start_list.elapsed();
        info!(duration_ms = duration_list.as_millis(), "Cloudflare delete_txt_record (LIST) API request completed");

        let res = res.map_err(|e| {
                let err = e.to_string();
                error!("Cloudflare request failed: {}", err);
                err
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Cloudflare API error ({}): {}", status, error_text);
            return Err(format!("Cloudflare API error ({}): {}", status, error_text));
        }

        let data: Value = res.json().await.map_err(|e| {
            let err = e.to_string();
            error!("Failed to parse Cloudflare response: {}", err);
            err
        })?;

        let records = data["result"].as_array().ok_or_else(|| {
            error!("Invalid response from Cloudflare: 'result' field missing or not an array");
            "Invalid response from Cloudflare".to_string()
        })?;

        if records.is_empty() {
            info!("No TXT record found to delete");
            return Ok(());
        }

        // Delete each matching record
        for record in records {
            let record_id = record["id"].as_str().ok_or_else(|| {
                error!("Record missing 'id' field");
                "Record missing 'id' field".to_string()
            })?;

            let delete_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_id
            );

            let start_del = std::time::Instant::now();
            let del_res = self.client.delete(&delete_url)
                .header(AUTHORIZATION, format!("Bearer {}", self.token))
                .header(CONTENT_TYPE, "application/json")
                .send()
                .await;
            let duration_del = start_del.elapsed();
            info!(duration_ms = duration_del.as_millis(), record_id = %record_id, "Cloudflare delete_txt_record (DELETE) API request completed");

            let del_res = del_res.map_err(|e| {
                    let err = e.to_string();
                    error!("Cloudflare delete request failed: {}", err);
                    err
                })?;

            if !del_res.status().is_success() {
                let status = del_res.status();
                let error_text = del_res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                error!("Cloudflare delete API error ({}): {}", status, error_text);
                return Err(format!("Cloudflare API error ({}): {}", status, error_text));
            }

            info!(record_id = %record_id, "Successfully deleted TXT record");
        }

        Ok(())
    }
}
