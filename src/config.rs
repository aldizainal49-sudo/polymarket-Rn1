use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub polymarket: PolymarketConfig,
    pub trading: TradingConfig,
    pub hft: HftConfig,
    pub hedging: HedgingConfig,
    pub zombie: ZombieConfig,
    pub farming: FarmingConfig,
    pub risk: RiskConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolymarketConfig {
    pub clob_api_url: String,
    pub gamma_api_url: String,
    pub ws_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub api_passphrase: String,
    pub private_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingConfig {
    pub max_markets: usize,
    pub scan_interval_ms: u64,
    pub min_liquidity: Decimal,
    pub mispricing_low_threshold: Decimal,
    pub mispricing_high_threshold: Decimal,
    pub min_ev_threshold: Decimal,
    pub maker_spread: Decimal,
    pub max_orders_per_market: usize,
    pub order_size: Decimal,
    pub hold_to_settlement: bool,
    pub max_active_positions: usize,
    pub paper_mode: bool,
    pub paper_balance: Decimal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HftConfig {
    pub enabled: bool,
    pub arbitrage_threshold: Decimal,
    pub latency_ms: u64,
    pub max_arb_size: Decimal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HedgingConfig {
    pub enabled: bool,
    pub max_correlation: Decimal,
    pub hedge_ratio: Decimal,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZombieConfig {
    pub enabled: bool,
    pub profit_take_threshold: Decimal,
    pub ignore_losses: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FarmingConfig {
    pub enabled: bool,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub farm_size: Decimal,
    pub expiry_window_minutes: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskConfig {
    pub max_position_value: Decimal,
    pub max_drawdown: Decimal,
    pub max_order_size: Decimal,
    pub min_price: Decimal,
    pub max_price: Decimal,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, anyhow::Error> {
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }
}
