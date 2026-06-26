use std::sync::Arc;
use rust_decimal::Decimal;
use chrono::Utc;
use tracing::info;
use crate::client::{ClobClient, GammaClient, OrderSide};
use crate::order::OrderManager;
use crate::config::FarmingConfig;

pub struct FarmingEngine {
    config: FarmingConfig,
    clob_client: Arc<ClobClient>,
    gamma_client: Arc<GammaClient>,
    order_manager: Arc<OrderManager>,
}

impl FarmingEngine {
    pub fn new(config: FarmingConfig, clob_client: Arc<ClobClient>, gamma_client: Arc<GammaClient>, order_manager: Arc<OrderManager>) -> Self {
        Self { config, clob_client, gamma_client, order_manager }
    }

    pub async fn run(&self) -> Result<(), anyhow::Error> {
        if !self.config.enabled { return Ok(()); }
        let markets = self.gamma_client.get_active_markets(500).await?;
        let now = Utc::now();
        for market in markets {
            let minutes_left = (market.end_date - now).num_minutes();
            if minutes_left > self.config.expiry_window_minutes || minutes_left <= 0 { continue; }
            if market.yes_price < self.config.max_price && market.yes_price > self.config.min_price {
                info!("👨‍🌾 FARMING: Buy YES at {:.2}¢ | {}", market.yes_price * Decimal::from(100), market.slug);
                self.order_manager.place_order(&market.market_id, OrderSide::Buy, market.yes_price, self.config.farm_size, false).await?;
            }
            if market.no_price < self.config.max_price && market.no_price > self.config.min_price {
                info!("👨‍🌾 FARMING: Buy NO at {:.2}¢ | {}", market.no_price * Decimal::from(100), market.slug);
                self.order_manager.place_order(&market.market_id, OrderSide::Buy, market.no_price, self.config.farm_size, false).await?;
            }
        }
        Ok(())
    }
}
