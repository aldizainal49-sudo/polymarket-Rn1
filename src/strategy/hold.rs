use std::collections::HashMap;
use std::sync::Arc;
use rust_decimal::Decimal;
use tracing::info;
use crate::client::ClobClient;
use crate::config::ZombieConfig;

/// Position structure for tracking open positions
#[derive(Debug, Clone)]
pub struct Position {
    pub market_id: String,
    pub token_id: String,
    pub side: String,
    pub size: Decimal,
    pub avg_price: Decimal,
    pub current_value: Decimal,
}

#[derive(Debug)]
pub struct HoldEngine {
    config: ZombieConfig,
    clob_client: Arc<ClobClient>,
    positions_cache: HashMap<String, Position>,
}

impl HoldEngine {
    pub fn new(config: ZombieConfig, clob_client: Arc<ClobClient>) -> Self {
        Self {
            config,
            clob_client,
            positions_cache: HashMap::new()
        }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // For now, just log that hold engine is running
        // In a real implementation, this would fetch positions from the API
        info!("Hold Engine running - tracking positions for zombie strategy");
        
        Ok(())
    }

    pub fn update_position(&mut self, position: Position) {
        let position_clone = position.clone();
        let current_price = if position.size > Decimal::ZERO {
            position.current_value / position.size
        } else {
            position.avg_price
        };
        
        let pnl_pct = if position.avg_price > Decimal::ZERO {
            (current_price - position.avg_price) / position.avg_price
        } else {
            Decimal::ZERO
        };
        
        self.positions_cache.insert(position.market_id.clone(), position);
        
        if pnl_pct > self.config.profit_take_threshold {
            info!("HOLDING BIG WIN: {} | +{:.2}%", position_clone.market_id, pnl_pct * Decimal::from(100));
        } else if pnl_pct < Decimal::ZERO && !self.config.ignore_losses {
            info!("HOLDING ZOMBIE: {} | {:.2}%", position_clone.market_id, pnl_pct * Decimal::from(100));
        }
    }
}
