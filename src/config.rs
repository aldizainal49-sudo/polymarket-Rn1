use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

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

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            max_markets: 10,
            scan_interval_ms: 1000,
            min_liquidity: dec!(1000),
            mispricing_low_threshold: dec!(0.05),
            mispricing_high_threshold: dec!(0.15),
            min_ev_threshold: dec!(0.5),
            maker_spread: dec!(0.02),
            max_orders_per_market: 5,
            order_size: dec!(10),
            hold_to_settlement: true,
            max_active_positions: 20,
            paper_mode: false,
            paper_balance: dec!(10000),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HftConfig {
    pub enabled: bool,
    pub arbitrage_threshold: Decimal,
    pub latency_ms: u64,
    pub max_arb_size: Decimal,
}

impl Default for HftConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            arbitrage_threshold: dec!(0.01),
            latency_ms: 10,
            max_arb_size: dec!(50),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HedgingConfig {
    pub enabled: bool,
    pub max_correlation: Decimal,
    pub hedge_ratio: Decimal,
}

impl Default for HedgingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_correlation: dec!(0.8),
            hedge_ratio: dec!(0.5),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZombieConfig {
    pub enabled: bool,
    pub profit_take_threshold: Decimal,
    pub ignore_losses: bool,
}

impl Default for ZombieConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            profit_take_threshold: dec!(0.1),
            ignore_losses: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FarmingConfig {
    pub enabled: bool,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub farm_size: Decimal,
    pub expiry_window_minutes: i64,
}

impl Default for FarmingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_price: dec!(0.1),
            max_price: dec!(0.9),
            farm_size: dec!(100),
            expiry_window_minutes: 60,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskConfig {
    pub max_position_value: Decimal,
    pub max_drawdown: Decimal,
    pub max_order_size: Decimal,
    pub min_price: Decimal,
    pub max_price: Decimal,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_value: dec!(1000),
            max_drawdown: dec!(0.1),
            max_order_size: dec!(100),
            min_price: dec!(0.01),
            max_price: dec!(0.99),
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, anyhow::Error> {
        let contents = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&contents)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            polymarket: PolymarketConfig {
                clob_api_url: "https://clob.polymarket.com/api".to_string(),
                gamma_api_url: "https://gamma-api.polymarket.com".to_string(),
                ws_url: "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string(),
                api_key: "".to_string(),
                api_secret: "".to_string(),
                api_passphrase: "".to_string(),
                private_key: "".to_string(),
            },
            trading: TradingConfig::default(),
            hft: HftConfig::default(),
            hedging: HedgingConfig::default(),
            zombie: ZombieConfig::default(),
            farming: FarmingConfig::default(),
            risk: RiskConfig::default(),
        }
    }
}
