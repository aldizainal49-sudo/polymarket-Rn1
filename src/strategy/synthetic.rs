use std::sync::Arc;
use rust_decimal::Decimal;
use tracing::info;
use crate::client::ClobClient;
use crate::client::types::OrderSide;
use crate::order::OrderManager;
use crate::strategy::hold::Position;

#[derive(Debug)]
pub struct SyntheticSellEngine {
    clob_client: Arc<ClobClient>,
    order_manager: Arc<OrderManager>,
}

impl SyntheticSellEngine {
    pub fn new(clob_client: Arc<ClobClient>, order_manager: Arc<OrderManager>) -> Self {
        Self {
            clob_client,
            order_manager
        }
    }

    pub async fn synthetic_sell(&self, position: &Position, opposing_price: Decimal) -> Result<(), anyhow::Error> {
        let side = match position.side.as_str() {
            "YES" | "Buy" => OrderSide::Sell,
            "NO" | "Sell" => OrderSide::Buy,
            _ => OrderSide::Sell
        };
        
        info!("SYNTHETIC SELL: {} @ {:.2}", position.market_id, opposing_price * Decimal::from(100));
        
        let mut om = self.order_manager.clone();
        let om_mut = Arc::make_mut(&mut om);
        om_mut.place_order(
            &position.market_id,
            side,
            opposing_price,
            position.size,
            false
        ).await?;
        
        Ok(())
    }
}
