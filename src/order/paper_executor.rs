use std::collections::HashMap;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::client::types::OrderSide;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperAccount {
    pub balance: Decimal,
    pub positions: HashMap<String, PaperPosition>,
    pub trade_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPosition {
    pub market_id: String,
    pub token_id: String,
    pub side: String,
    pub shares: Decimal,
    pub avg_price: Decimal,
    pub entry_value: Decimal,
}
