use std::sync::Arc;
use std::collections::HashMap;
use rust_decimal::Decimal;
use tracing::info;

use crate::client::{ClobClient, Order, OrderSide, OrderStatus};
use crate::order::paper_executor::PaperExecutor;

pub struct OrderManager {
    clob_client: Arc<ClobClient>,
    paper_executor: Option<PaperExecutor>,
    active_orders: HashMap<String, Order>,
}

impl OrderManager {
    pub fn new_real(clob_client: Arc<ClobClient>) -> Self {
        Self { clob_client, paper_executor: None, active_orders: HashMap::new() }
    }

    pub fn new_paper(clob_client: Arc<ClobClient>, paper_executor: PaperExecutor) -> Self {
        Self { clob_client, paper_executor: Some(paper_executor), active_orders: HashMap::new() }
    }

    pub async fn place_order(&mut self, token_id: &str, side: OrderSide, price: Decimal, size: Decimal, post_only: bool) -> Result<Order, anyhow::Error> {
        if let Some(paper) = &mut self.paper_executor {
            let market_id = format!("paper_{}", token_id);
            match side {
                OrderSide::Buy => { paper.buy(&market_id, token_id, side, price, size)?; }
                OrderSide::Sell => { paper.sell(&market_id, token_id, side, price, size)?; }
            }
            let dummy = Order { order_id: format!("paper_{}_{}", token_id, chrono::Utc::now().timestamp()), market_id, token_id: token_id.to_string(), side, price, size, filled_size: size, status: OrderStatus::Filled };
            return Ok(dummy);
        }
        let order = self.clob_client.place_limit_order(token_id, side, price, size, post_only).await?;
        self.active_orders.insert(order.order_id.clone(), order.clone());
        info!("✅ [REAL] Order placed: {} @ {:.2}¢", order.order_id, price * Decimal::from(100));
        Ok(order)
    }

    pub async fn cancel_all(&mut self) -> Result<(), anyhow::Error> {
        if self.paper_executor.is_some() { self.active_orders.clear(); return Ok(()); }
        self.clob_client.cancel_all_orders().await?;
        self.active_orders.clear();
        Ok(())
    }

    pub fn get_paper_executor(&mut self) -> Option<&mut PaperExecutor> {
        self.paper_executor.as_mut()
    }
}
