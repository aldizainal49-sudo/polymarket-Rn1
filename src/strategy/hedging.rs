use std::sync::Arc;
use rust_decimal::Decimal;
use tracing::info;
use crate::client::{ClobClient, MarketData};
use crate::order::OrderManager;
use crate::config::HedgingConfig;

pub struct HedgingEngine {
    config: HedgingConfig,
    clob_client: Arc<ClobClient>,
    order_manager: Arc<OrderManager>,
}

impl HedgingEngine {
    pub fn new(config: HedgingConfig, clob_client: Arc<ClobClient>, order_manager: Arc<OrderManager>) -> Self {
        Self { config, clob_client, order_manager }
    }

    pub async fn run(&self, markets: &[MarketData]) -> Result<(), anyhow::Error> {
        if !self.config.enabled { return Ok(()); }
        for (i, a) in markets.iter().enumerate() {
            for b in markets.iter().skip(i + 1) {
                let slug_a = a.slug.to_lowercase();
                let slug_b = b.slug.to_lowercase();
                let teams_a: Vec<&str> = slug_a.split('-').collect();
                let teams_b: Vec<&str> = slug_b.split('-').collect();
                for team_a in &teams_a {
                    for team_b in &teams_b {
                        if team_a == team_b && team_a.len() > 3 {
                            let deviation = (a.yes_price - b.yes_price).abs();
                            if deviation > Decimal::from(10) / Decimal::from(100) {
                                info!("🛡️ HEDGE: {} vs {} | Deviasi: {:.2}%", a.slug, b.slug, deviation * Decimal::from(100));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
