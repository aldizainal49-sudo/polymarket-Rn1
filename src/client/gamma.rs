use reqwest::Client;
use serde_json::Value;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use crate::client::types::MarketData;

#[derive(Debug)]
pub struct GammaClient {
    client: Client,
    base_url: String,
}

impl GammaClient {
    pub fn new(base_url: &str) -> Self {
        Self { client: Client::new(), base_url: base_url.to_string() }
    }

    pub async fn get_active_markets(&self, limit: usize) -> Result<Vec<MarketData>, anyhow::Error> {
        let url = format!("{}/markets?active=true&limit={}", self.base_url, limit);
        let response = self.client.get(&url).send().await?;
        let data: Value = response.json().await?;

        let mut markets = Vec::new();
        
        if let Some(markets_array) = data["data"].as_array() {
            for m in markets_array {
                let category = m["category"].as_str().unwrap_or("");
                if !["sports", "Sports", "sport"].contains(&category) {
                    continue;
                }
                
                let volume = Decimal::from_str_exact(m["volume"].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO);
                if volume < Decimal::from(1000) {
                    continue;
                }
                
                let yes_price = Decimal::from_str_exact(m["yes_price"].as_str().unwrap_or("0.5"))
                    .unwrap_or(Decimal::from(50) / Decimal::from(100));
                let no_price = Decimal::from_str_exact(m["no_price"].as_str().unwrap_or("0.5"))
                    .unwrap_or(Decimal::from(50) / Decimal::from(100));
                
                let end_date = m["end_date"].as_str().and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(s)
                        .map(|d| d.with_timezone(&chrono::Utc))
                        .ok()
                }).unwrap_or(chrono::Utc::now());

                markets.push(MarketData {
                    market_id: m["id"].as_str().unwrap_or("").to_string(),
                    slug: m["slug"].as_str().unwrap_or("").to_string(),
                    question: m["question"].as_str().unwrap_or("").to_string(),
                    yes_price,
                    no_price,
                    volume,
                    liquidity: Decimal::from_str_exact(m["liquidity"].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO),
                    active: true,
                    end_date,
                });
            }
        }
        
        Ok(markets)
    }

    pub async fn get_market_details(&self, market_id: &str) -> Result<MarketData, anyhow::Error> {
        let url = format!("{}/markets/{}", self.base_url, market_id);
        let response = self.client.get(&url).send().await?;
        let data: Value = response.json().await?;

        let m = data["data"].as_object().ok_or_else(|| anyhow::anyhow!("Market not found"))?;
        
        let yes_price = Decimal::from_str_exact(m["yes_price"].as_str().unwrap_or("0.5"))
            .unwrap_or(Decimal::from(50) / Decimal::from(100));
        let no_price = Decimal::from_str_exact(m["no_price"].as_str().unwrap_or("0.5"))
            .unwrap_or(Decimal::from(50) / Decimal::from(100));
        
        let end_date = m["end_date"].as_str().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|d| d.with_timezone(&chrono::Utc))
                .ok()
        }).unwrap_or(chrono::Utc::now());

        Ok(MarketData {
            market_id: m["id"].as_str().unwrap_or("").to_string(),
            slug: m["slug"].as_str().unwrap_or("").to_string(),
            question: m["question"].as_str().unwrap_or("").to_string(),
            yes_price,
            no_price,
            volume: Decimal::from_str_exact(m["volume"].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO),
            liquidity: Decimal::from_str_exact(m["liquidity"].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO),
            active: m["active"].as_bool().unwrap_or(false),
            end_date,
        })
    }
}
