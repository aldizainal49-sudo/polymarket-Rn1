use rust_decimal::Decimal;
use crate::client::types::MarketData;
use crate::config::TradingConfig;

#[derive(Debug)]
pub struct MispricingEngine {
    config: TradingConfig,
}

impl MispricingEngine {
    pub fn new(config: &TradingConfig) -> Self {
        Self { config: config.clone() }
    }

    pub fn calculate_ev(&self, market: &MarketData) -> Option<MispricingSignal> {
        let yes_price = market.yes_price;
        let sum = yes_price + market.no_price;
        
        if sum < Decimal::ONE && Decimal::ONE - sum >= self.config.min_ev_threshold {
            return Some(MispricingSignal {
                market_id: market.market_id.clone(),
                signal_type: SignalType::Arbitrage,
                yes_price,
                no_price: market.no_price,
                ev: Decimal::ONE - sum,
                recommended_action: Action::BuyBoth,
                size: self.config.order_size
            });
        }
        
        if yes_price < self.config.mispricing_low_threshold {
            let estimated = self.estimate_true_probability(market);
            if estimated > yes_price + self.config.min_ev_threshold {
                return Some(MispricingSignal {
                    market_id: market.market_id.clone(),
                    signal_type: SignalType::MispricingLow,
                    yes_price,
                    no_price: market.no_price,
                    ev: estimated - yes_price,
                    recommended_action: Action::BuyYes,
                    size: self.config.order_size
                });
            }
        }
        
        if yes_price > self.config.mispricing_high_threshold {
            let estimated = self.estimate_true_probability(market);
            let no_implied = Decimal::ONE - yes_price;
            let no_estimated = Decimal::ONE - estimated;
            if no_estimated > no_implied + self.config.min_ev_threshold {
                return Some(MispricingSignal {
                    market_id: market.market_id.clone(),
                    signal_type: SignalType::MispricingHigh,
                    yes_price,
                    no_price: market.no_price,
                    ev: no_estimated - no_implied,
                    recommended_action: Action::BuyNo,
                    size: self.config.order_size
                });
            }
        }
        
        None
    }

    fn estimate_true_probability(&self, market: &MarketData) -> Decimal {
        let base = market.yes_price;
        let liquidity_factor = if market.liquidity > Decimal::from(10000) {
            Decimal::ONE
        } else if market.liquidity > Decimal::from(1000) {
            Decimal::from(2)
        } else {
            Decimal::from(5)
        };
        let adjustment = (Decimal::from(50) / Decimal::from(100) - base) / liquidity_factor;
        (base + adjustment).max(Decimal::ZERO).min(Decimal::ONE)
    }
}

#[derive(Debug, Clone)]
pub struct MispricingSignal {
    pub market_id: String,
    pub signal_type: SignalType,
    pub yes_price: Decimal,
    pub no_price: Decimal,
    pub ev: Decimal,
    pub recommended_action: Action,
    pub size: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    Arbitrage,
    MispricingLow,
    MispricingHigh
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    BuyYes,
    BuyNo,
    BuyBoth,
    None
}