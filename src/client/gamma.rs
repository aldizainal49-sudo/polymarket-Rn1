use reqwest::Client;
use serde_json::Value;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use crate::client::MarketData;

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

        let markets = data["data"].as_array().unwrap_or(&vec![])
            .iter().filter_map(|m| {
                let category = m["category"].as_str().unwrap_or("");
                if !["sports", "Sports", "sport"].contains(&category) { return None; }
                let volume = Decimal::from_str(m["volume"].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO);
                if volume < Decimal::from(1000) { return None; }
                Some(MarketData {
                    market_id: m["id"].as_str().unwrap_or("").to_string(),
                    slug: m["slug"].as_str().unwrap_or("").to_string(),
                    question: m["question"].as_str().unwrap_or("").to_string(),
                    yes_price: Decimal::from_str(m["yes_price"].as_str().unwrap_or("0.5")).unwrap_or(Decimal::from(50) / Decimal::from(100)),
                    no_price: Decimal::from_str(m["no_price"].as_str().unwrap_or("0.5")).unwrap_or(Decimal::from(50) / Decimal::from(100)),
                    volume,
                    liquidity: Decimal::from_str(m["liquidity"].as_str().unwrap_or("0")).unwrap_or(Decimal::ZERO),
                    active: true,
                    end_date: chrono::DateTime::parse_from_rfc3339(m["end_date"].as_str().unwrap_or("")).map(|d| d.with_timezone(&chrono::Utc)).unwrap_or(chrono::Utc::now()),
                })
            }).collect();
        Ok(markets)
    }
}
