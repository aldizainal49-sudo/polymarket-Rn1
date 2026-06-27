use std::collections::HashMap;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;
use anyhow::Result;

use crate::client::types::{OrderSide, MarketData};
use crate::strategy::mispricing::MispricingEngine;
use crate::config::TradingConfig;

#[derive(Debug, Deserialize)]
pub struct HistoricalMarket {
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    pub market_id: String,
    pub slug: String,
    pub yes_price: Decimal,
    pub no_price: Decimal,
    pub volume: Decimal,
    pub resolved: bool,
    pub outcome_yes: bool,
}

impl From<&HistoricalMarket> for MarketData {
    fn from(hm: &HistoricalMarket) -> Self {
        MarketData {
            market_id: hm.market_id.clone(),
            slug: hm.slug.clone(),
            question: format!("Historical: {}", hm.slug),
            yes_price: hm.yes_price,
            no_price: hm.no_price,
            volume: hm.volume,
            liquidity: Decimal::ZERO,
            active: false,
            end_date: hm.timestamp,
        }
    }
}

pub struct BacktestTrade {
    pub timestamp: DateTime<Utc>,
    pub market_id: String,
    pub side: OrderSide,
    pub price: Decimal,
    pub size: Decimal,
    pub pnl: Decimal,
}

pub struct BacktestEngine {
    config: TradingConfig,
    mispricing_engine: MispricingEngine,
    balance: Decimal,
    positions: HashMap<String, (OrderSide, Decimal, Decimal)>,
    trades: Vec<BacktestTrade>,
    initial_balance: Decimal,
}

impl BacktestEngine {
    pub fn new(config: TradingConfig, initial_balance: Decimal) -> Self {
        Self {
            mispricing_engine: MispricingEngine::new(&config),
            balance: initial_balance,
            positions: HashMap::new(),
            trades: Vec::new(),
            initial_balance,
            config,
        }
    }

    pub async fn run(&mut self, data_path: &str) -> Result<()> {
        info!("Running Backtest with balance: ${:.2}", self.initial_balance);
        let markets = self.load_historical_data(data_path)?;

        for market in markets {
            let market_data: MarketData = (&market).into();
            if let Some(_signal) = self.mispricing_engine.calculate_ev(&market_data) {
                let price = market.yes_price;
                let size = self.config.order_size;
                let cost = price * size;
                if cost > self.balance {
                    continue;
                }

                self.balance -= cost;
                self.positions.insert(market.market_id.clone(), (OrderSide::Buy, size, price));
                self.trades.push(BacktestTrade {
                    timestamp: market.timestamp,
                    market_id: market.market_id.clone(),
                    side: OrderSide::Buy,
                    price,
                    size,
                    pnl: Decimal::ZERO,
                });
                info!("Backtest BUY: {} @ {:.2}", market.slug, price * Decimal::from(100));
            }

            if market.resolved {
                if let Some((_side, size, avg_price)) = self.positions.remove(&market.market_id) {
                    let payout = if market.outcome_yes {
                        size * Decimal::ONE
                    } else {
                        Decimal::ZERO
                    };
                    let pnl = payout - (avg_price * size);
                    self.balance += payout;
                    if let Some(last) = self.trades.last_mut() {
                        last.pnl = pnl;
                    }
                    info!("Settle: {} | PnL: ${:.2}", market.slug, pnl);
                }
            }
        }

        self.print_summary();
        Ok(())
    }

    fn load_historical_data(&self, path: &str) -> Result<Vec<HistoricalMarket>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut rdr = csv::Reader::from_reader(reader);
        let mut markets = Vec::new();
        for result in rdr.deserialize() {
            markets.push(result?);
        }
        Ok(markets)
    }

    fn print_summary(&self) {
        let total_pnl = self.balance - self.initial_balance;
        let win_count = self.trades.iter().filter(|t| t.pnl > Decimal::ZERO).count();
        let total = self.trades.len();
        info!("===== BACKTEST SUMMARY =====");
        info!("   Initial Balance: ${:.2}", self.initial_balance);
        info!("   Final Balance:   ${:.2}", self.balance);
        info!("   Total PnL:       ${:.2} ({:.2}%)", total_pnl, (total_pnl / self.initial_balance) * Decimal::from(100));
        info!("   Total Trades:    {}", total);
        info!("   Wins:            {}", win_count);
        if total > 0 {
            info!("   Win Rate:        {:.2}%", (win_count as f64 / total as f64) * 100.0);
        }
        info!("==============================");
    }
}
