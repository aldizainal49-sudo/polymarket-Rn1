use std::sync::Arc;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use tokio::sync::mpsc;
use tracing::info;
use dashmap::DashMap;

use crate::client::websocket::PriceUpdate;
use crate::client::ClobClient;
use crate::order::OrderManager;
use crate::config::HftConfig;

pub struct HftEngine {
    config: HftConfig,
    clob_client: Arc<ClobClient>,
    order_manager: Arc<OrderManager>,
    price_cache: Arc<DashMap<String, Decimal>>,
}

impl HftEngine {
    pub fn new(config: HftConfig, clob_client: Arc<ClobClient>, order_manager: Arc<OrderManager>) -> Self {
        Self { config, clob_client, order_manager, price_cache: Arc::new(DashMap::new()) }
    }

    pub async fn run(&self, mut price_rx: mpsc::Receiver<PriceUpdate>) -> Result<(), anyhow::Error> {
        if !self.config.enabled { return Ok(()); }
        info!("⚡ HFT Engine started");
        while let Some(update) = price_rx.recv().await {
            self.price_cache.insert(update.market_id.clone(), update.price);
        }
        Ok(())
    }
}
