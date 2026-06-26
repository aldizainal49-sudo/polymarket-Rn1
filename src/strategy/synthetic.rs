use std::sync::Arc;
use rust_decimal::Decimal;
use tracing::info;
use crate::client::{ClobClient, Position, OrderSide};
use crate::order::OrderManager;

pub struct SyntheticSellEngine {
    clob_client: Arc<ClobClient>,
    order_manager: Arc<OrderManager>,
}

impl SyntheticSellEngine {
    pub fn new(clob_client: Arc<ClobClient>, order_manager: Arc<OrderManager>) -> Self {
        Self { clob_client, order_manager }
    }

    pub async fn synthetic_sell(&self, position: &Position, opposing_price: Decimal) -> Result<(), anyhow::Error> {
        let side = match position.side { OrderSide::Buy => OrderSide::Sell, OrderSide::Sell => OrderSide::Buy };
        info!("🔄 SYNTHETIC SELL: {} @ {:.2}¢", position.market_id, opposing_price * Decimal::from(100));
        self.order_manager.place_order(&position.market_id, side, opposing_price, position.size, false).await?;
        Ok(())
    }
}
