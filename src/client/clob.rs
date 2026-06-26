use std::sync::Arc;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde_json::json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use chrono::Utc;

use crate::config::Config;
use crate::client::{Order, OrderSide, OrderStatus, OrderBook};

type HmacSha256 = Hmac<Sha256>;

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

        let bids = data["bids"].as_array().unwrap_or(&vec![])
            .iter().map(|b| (Decimal::from_str(b[0].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO), Decimal::from_str(b[1].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO))).collect();
        let asks = data["asks"].as_array().unwrap_or(&vec![])
            .iter().map(|b| (Decimal::from_str(b[0].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO), Decimal::from_str(b[1].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO))).collect();

        let best_bid = bids.first().map(|(p, _)| *p).unwrap_or(Decimal::ZERO);
        let best_ask = asks.first().map(|(p, _)| *p).unwrap_or(Decimal::ONE);
        
        Ok(OrderBook {
            market_id: data["market"].as_str().unwrap_or("").to_string(),
            token_id: token_id.to_string(),
            best_bid,
            best_ask,
            bids,
            asks,
            mid_price: (best_bid + best_ask) / Decimal::from(2),
            spread: best_ask - best_bid,
        })
    }

    pub async fn place_limit_order(&self, token_id: &str, side: OrderSide, price: Decimal, size: Decimal, post_only: bool) -> Result<Order, anyhow::Error> {
        let path = "/order";
        let body = json!({
            "token_id": token_id,
            "side": match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" },
            "price": price.to_string(),
            "size": size.to_string(),
            "post_only": post_only,
        }).to_string();
        
        let signature = self.sign_request("POST", path, &body);
        let timestamp = Utc::now().timestamp().to_string();

        let response = self.client.post(&format!("{}{}", self.base_url, path))
            .header("POLY_API_KEY", &self.api_key)
            .header("POLY_SIGNATURE", signature)
            .header("POLY_TIMESTAMP", timestamp)
            .header("POLY_PASSPHRASE", &self.passphrase)
            .header("Content-Type", "application/json")
            .body(body).send().await?;

        let data: serde_json::Value = response.json().await?;
        Ok(Order {
            order_id: data["order_id"].as_str().unwrap_or("").to_string(),
            market_id: data["market"].as_str().unwrap_or("").to_string(),
            token_id: token_id.to_string(),
            side,
            price,
            size,
            filled_size: Decimal::ZERO,
            status: OrderStatus::Open,
        })
    }

    pub async fn cancel_all_orders(&self) -> Result<(), anyhow::Error> {
        let path = "/orders";
        let signature = self.sign_request("DELETE", path, "");
        let timestamp = Utc::now().timestamp().to_string();
        self.client.delete(&format!("{}{}", self.base_url, path))
            .header("POLY_API_KEY", &self.api_key)
            .header("POLY_SIGNATURE", signature)
            .header("POLY_TIMESTAMP", timestamp)
            .header("POLY_PASSPHRASE", &self.passphrase)
            .send().await?;
        Ok(())
    }
}
