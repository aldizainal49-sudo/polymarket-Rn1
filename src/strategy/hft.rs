use std::sync::Arc;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use tokio::sync::mpsc;
use tracing::info;
use dashmap::DashMap;

use crate::client::ClobClient;
use crate::client::websocket::PriceUpdate;
use crate::order::OrderManager;
use crate::config::HftConfig;

#[derive(Debug)]
pub struct HftEngine {
    config: HftConfig,
    clob_client: Arc<ClobClient>,
    order_manager: Arc<OrderManager>,
    price_cache: Arc<DashMap<String, Decimal>>,
}

impl HftEngine {
    pub fn new(config: HftConfig, clob_client: Arc<ClobClient>, order_manager: Arc<OrderManager>) -> Self {
        Self {
            config,
            clob_client,
            order_manager,
            price_cache: Arc::new(DashMap::new())
        }
    }

    pub async fn run(&self, mut price_rx: mpsc::Receiver<PriceUpdate>) -> Result<(), anyhow::Error> {
        if !self.config.enabled {
            return Ok(());
        }
        info!("HFT Engine started");
        
        while let Some(update) = price_rx.recv().await {
            self.price_cache.insert(update.token_id.clone(), update.price);
            
            // Check for arbitrage opportunities
            if self.config.arbitrage_threshold > Decimal::ZERO {
                self.check_arbitrage(&update).await?;
            }
        }
        Ok(())
    }

    async fn check_arbitrage(&self, update: &PriceUpdate) -> Result<(), anyhow::Error> {
        // Implement arbitrage logic here
        // For now, just log the price update
        info!("Price update: {} @ {}", update.token_id, update.price);
        Ok(())
    }
}
