use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Order side for trading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Current status of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

/// Represents an order in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub market_id: String,
    pub token_id: String,
    pub side: OrderSide,
    pub price: Decimal,
    pub size: Decimal,
    pub filled_size: Decimal,
    pub status: OrderStatus,
}

impl Order {
    pub fn remaining_size(&self) -> Decimal {
        self.size - self.filled_size
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, OrderStatus::Open | OrderStatus::PartiallyFilled)
    }
}

/// Represents an order book with bids and asks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub market_id: String,
    pub token_id: String,
    pub best_bid: Decimal,
    pub best_ask: Decimal,
    pub bids: VecDeque<(Decimal, Decimal)>, // (price, size)
    pub asks: VecDeque<(Decimal, Decimal)>, // (price, size)
    pub mid_price: Decimal,
    pub spread: Decimal,
}

impl OrderBook {
    pub fn new(market_id: String, token_id: String) -> Self {
        Self {
            market_id,
            token_id,
            best_bid: Decimal::ZERO,
            best_ask: Decimal::ZERO,
            bids: VecDeque::new(),
            asks: VecDeque::new(),
            mid_price: Decimal::ZERO,
            spread: Decimal::ZERO,
        }
    }

    pub fn depth(&self, levels: usize) -> (Vec<(Decimal, Decimal)>, Vec<(Decimal, Decimal)>) {
        let bids: Vec<_> = self.bids.iter().take(levels).cloned().collect();
        let asks: Vec<_> = self.asks.iter().take(levels).cloned().collect();
        (bids, asks)
    }

    pub fn total_liquidity(&self) -> Decimal {
        let bids_liquidity: Decimal = self.bids.iter().map(|(_, size)| *size).sum();
        let asks_liquidity: Decimal = self.asks.iter().map(|(_, size)| *size).sum();
        bids_liquidity + asks_liquidity
    }
}

/// Market data structure for trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub market_id: String,
    pub slug: String,
    pub question: String,
    pub yes_price: Decimal,
    pub no_price: Decimal,
    pub volume: Decimal,
    pub liquidity: Decimal,
    pub active: bool,
    pub end_date: chrono::DateTime<chrono::Utc>,
}

impl MarketData {
    pub fn implied_probability(&self) -> Decimal {
        self.yes_price
    }

    pub fn is_expired(&self) -> bool {
        self.end_date <= chrono::Utc::now()
    }
}

/// Price tick for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTick {
    pub market_id: String,
    pub token_id: String,
    pub price: Decimal,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub token_id: String,
    pub contract_address: String,
    pub decimals: u8,
    pub symbol: String,
}
