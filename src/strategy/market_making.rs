use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use crate::client::{OrderBook, OrderSide};
use crate::config::TradingConfig;

pub struct MarketMakingEngine {
    config: TradingConfig,
}

impl MarketMakingEngine {
    pub fn new(config: &TradingConfig) -> Self { Self { config: config.clone() } }

    pub fn generate_maker_orders(&self, order_book: &OrderBook) -> Vec<MakerOrder> {
        let mut orders = Vec::new();
        let mid = order_book.mid_price;
        let spread = self.config.maker_spread;
        let yes_bid = (mid - spread).max(Decimal::ZERO);
        if yes_bid >= self.config.min_price {
            orders.push(MakerOrder { token_id: order_book.token_id.clone(), side: OrderSide::Buy, price: yes_bid, size: self.config.order_size, post_only: true });
        }
        orders
    }
}

#[derive(Debug, Clone)]
pub struct MakerOrder {
    pub token_id: String,
    pub side: OrderSide,
    pub price: Decimal,
    pub size: Decimal,
    pub post_only: bool,
}
