use std::collections::HashMap;
use std::sync::Arc;
use rust_decimal::Decimal;
use tracing::info;
use crate::client::{ClobClient, Position};
use crate::config::ZombieConfig;

pub struct HoldEngine {
    config: ZombieConfig,
    clob_client: Arc<ClobClient>,
    positions_cache: HashMap<String, Position>,
}

impl HoldEngine {
    pub fn new(config: ZombieConfig, clob_client: Arc<ClobClient>) -> Self {
        Self { config, clob_client, positions_cache: HashMap::new() }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        if !self.config.enabled { return Ok(()); }
        let positions = self.clob_client.get_positions().await?;
        for pos in positions {
            let current_price = if pos.size > Decimal::ZERO { pos.current_value / pos.size } else { pos.avg_price };
            let pnl_pct = if pos.avg_price > Decimal::ZERO { (current_price - pos.avg_price) / pos.avg_price } else { Decimal::ZERO };
            self.positions_cache.insert(pos.market_id.clone(), pos);
            if pnl_pct > Decimal::from(5) / Decimal::from(100) {
                info!("🧟 HOLDING BIG WIN: {} | +{:.2}%", pos.market_id, pnl_pct * Decimal::from(100));
            } else if pnl_pct < Decimal::ZERO {
                info!("🧟 HOLDING ZOMBIE: {} | {:.2}%", pos.market_id, pnl_pct * Decimal::from(100));
            }
        }
        Ok(())
    }
}
