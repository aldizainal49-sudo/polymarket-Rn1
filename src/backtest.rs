use std::collections::HashMap;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;
use anyhow::Result;

use crate::client::types::{OrderSide, MarketData};
use crate::strategy::mispricing::MispricingEngine;
use crate::config::TradingConfig;

#[derive(Debug, Deserialize)]
pub struct HistoricalMarket {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub market_id: String,
    pub slug: String,
    pub yes_price: Decimal,
    pub no_price: Decimal,
    pub volume: Decimal,
    pub resolved: bool,
    pub outcome_yes: bool,
}

impl From<&HistoricalMarket> for MarketData {
    fn from(hm: &HistoricalMarket) -> Self {
        MarketData {
            market_id: hm.market_id.clone(),
            slug: hm.slug.clone(),
            question: format!("Historical: {}", hm.slug),
            yes_price: hm.yes_price,
            no_price: hm.no_price,
            volume: hm.volume,
            liquidity: Decimal::ZERO,
            active: false,
            end_date: hm.timestamp,
        }
    }
}
