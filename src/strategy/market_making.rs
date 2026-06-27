use rust_decimal::Decimal;
use crate::client::types::{OrderBook, OrderSide};
use crate::config::TradingConfig;

#[derive(Debug)]
pub struct MarketMakingEngine {
    config: TradingConfig,
}

impl MarketMakingEngine {
    pub fn new(config: &TradingConfig) -> Self {
        Self { config: config.clone() }
    }

    pub fn generate_maker_orders(&self, order_book: &OrderBook) -> Vec<MakerOrder> {
        let mut orders = Vec::new();
        let mid = order_book.mid_price;
        let spread = self.config.maker_spread;
        
        let yes_bid = (mid - spread).max(Decimal::ZERO);
        if yes_bid > Decimal::ZERO {
            orders.push(MakerOrder {
                token_id: order_book.token_id.clone(),
                side: OrderSide::Buy,
                price: yes_bid,
                size: self.config.order_size,
                post_only: true
            });
        }
        
        let yes_ask = (mid + spread).min(Decimal::ONE);
        if yes_ask < Decimal::ONE {
            orders.push(MakerOrder {
                token_id: order_book.token_id.clone(),
                side: OrderSide::Sell,
                price: yes_ask,
                size: self.config.order_size,
                post_only: true
            });
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