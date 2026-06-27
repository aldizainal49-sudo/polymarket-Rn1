use std::collections::VecDeque;
use reqwest::Client;
use rust_decimal::Decimal;
use serde_json::json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use chrono::Utc;

use crate::config::Config;
use crate::client::types::{Order, OrderSide, OrderStatus, OrderBook};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct ClobClient {
    client: Client,
    base_url: String,
    api_key: String,
    api_secret: String,
    passphrase: String,
}

impl ClobClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            base_url: config.polymarket.clob_api_url.clone(),
            api_key: config.polymarket.api_key.clone(),
            api_secret: config.polymarket.api_secret.clone(),
            passphrase: config.polymarket.api_passphrase.clone(),
        }
    }

    fn sign_request(&self, method: &str, path: &str, body: &str) -> String {
        let timestamp = Utc::now().timestamp().to_string();
        let message = format!("{}{}{}{}", timestamp, method, path, body);
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes()).expect("HMAC key");
        mac.update(message.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub async fn get_order_book(&self, token_id: &str) -> Result<OrderBook, anyhow::Error> {
        let url = format!("{}/book?token_id={}", self.base_url, token_id);
        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        let mut bids = VecDeque::new();
        if let Some(bids_array) = data["bids"].as_array() {
            for b in bids_array {
                let price = Decimal::from_str_exact(b[0].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO);
                let size = Decimal::from_str_exact(b[1].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO);
                bids.push_back((price, size));
            }
        }

        let mut asks = VecDeque
