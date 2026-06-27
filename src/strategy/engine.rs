use std::sync::Arc;
use tokio::time::{self, Duration};
use tracing::info;

use crate::config::Config;
use crate::client::ClobClient;
use crate::client::GammaClient;
use crate::client::websocket::{WebSocketManager, PriceUpdate};
use crate::order::OrderManager;
use crate::order::paper_executor::PaperExecutor;
use crate::ai_client::AIClient;
use crate::logger::SheetLogger;
use crate::strategy::mispricing::MispricingEngine;
use crate::strategy::market_making::MarketMakingEngine;
use crate::strategy::hft::HftEngine;
use crate::strategy::hedging::HedgingEngine;
use crate::strategy::hold::HoldEngine;
use crate::strategy::farming::FarmingEngine;
use crate::client::sportmonks::SportmonksClient;
use crate::client::sportsdataio::SportsDataIOClient;
use crate::client::sportradar::SportradarClient;
use crate::client::pmxt_ws_pool::PmxtWebSocketPool;

#[derive(Debug)]
pub struct TradingEngine {
    config: Config,
    clob_client: Arc<ClobClient>,
    gamma_client: Arc<GammaClient>,
    order_manager: Arc<OrderManager>,
    ai_client: Option<AIClient>,
    logger: Option<SheetLogger>,
    sportmonks_client: Option<SportmonksClient>,
    sportsdataio_client: Option<SportsDataIOClient>,
    sportradar_client: Option<SportradarClient>,
    pmxt_pool: Option<PmxtWebSocketPool>,
    mispricing_engine: MispricingEngine,
    market_making_engine: MarketMakingEngine,
    hft_engine: Arc<HftEngine>,
    hedging_engine: Arc<HedgingEngine>,
    hold_engine: Arc<HoldEngine>,
    farming_engine: Arc<FarmingEngine>,
}

impl TradingEngine {
    pub async fn new(
        config: Config,
        ai_client: Option<AIClient>,
        logger: Option<SheetLogger>,
        sportmonks_client: Option<SportmonksClient>,
        sportsdataio_client: Option<SportsDataIOClient>,
        sportradar_client: Option<SportradarClient>,
        pmxt_pool: Option<PmxtWebSocketPool>,
    ) -> Result<Self, anyhow::Error> {
        let clob_client = Arc::new(ClobClient::new(&config));
        let gamma_client = Arc::new(GammaClient::new(&config.polymarket.gamma_api_url));

        let order_manager = if config.trading.paper_mode {
            let paper = PaperExecutor::new(config.trading.paper_balance, "paper_state.json");
            Arc::new(OrderManager::new_paper(clob_client.clone(), paper))
        } else {
            Arc::new(OrderManager::new_real(clob_client.clone()))
        };

        let mispricing_engine = MispricingEngine::new(&config.trading);
        let market_making_engine = MarketMakingEngine::new(&config.trading);
        let hft_engine = Arc::new(HftEngine::new(config.hft.clone(), clob_client.clone(), order_manager.clone()));
        let hedging_engine = Arc::new(HedgingEngine::new(config.hedging.clone(), clob_client.clone(), order_manager.clone()));
        let hold_engine = Arc::new(HoldEngine::new(config.zombie.clone(), clob_client.clone()));
        let farming_engine = Arc::new(FarmingEngine::new(config.farming.clone(), clob_client.clone(), gamma_client.clone(), order_manager.clone()));

        Ok(Self {
            config,
            clob_client,
            gamma_client,
            order_manager,
            ai_client,
            logger,
            sportmonks_client,
            sportsdataio_client,
            sportradar_client,
            pmxt_pool,
            mispricing_engine,
            market_making_engine,
            hft_engine,
            hedging_engine,
            hold_engine,
            farming_engine,
        })
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        info!("Starting RN1 Trading Engine v2.0");
        info!("Strategies: Mispricing | MarketMaking | HFT | Hedging | Hold | Farming");

        let (price_tx, _price_rx) = tokio::sync::mpsc::channel::<PriceUpdate>(10000);
        let ws_manager = WebSocketManager::new(Arc::new(self.config.clone()), price_tx);
        tokio::spawn(async move {
            if let Err(e) = ws_manager.run().await {
                eprintln!("WS error: {}", e);
            }
        });

        let _hft_engine = self.hft_engine.clone();
        let _hedging_engine = self.hedging_engine.clone();
        let _hold_engine = self.hold_engine.clone();
        let _farming_engine = self.farming_engine.clone();
        let order_manager = self.order_manager.clone();
        let clob = self.clob_client.clone();
        let gamma = self.gamma_client.clone();
        let market_making_engine = MarketMakingEngine::new(&self.config.trading);

        let scan_interval = self.config.trading.scan_interval_ms;
        let main_task = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(scan_interval));
            loop {
                interval.tick().await;
                if let Ok(markets) = gamma.get_active_markets(100).await {
                    for market in markets {
                        if let Ok(order_book) = clob.get_order_book(&market.market_id).await {
                            let orders = market_making_engine.generate_maker_orders(&order_book);
                            for order in orders {
                                let mut om = order_manager.clone();
                                let om_mut = Arc::make_mut(&mut om);
                                let _ = om_mut.place_order(&order.token_id, order.side, order.price, order.size, order.post_only).await;
                            }
                        }
                    }
                }
            }
        });

        tokio::try_join!(main_task)?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), anyhow::Error> {
        info!("Shutting down...");
        self.clob_client.cancel_all_orders().await?;
        if let Some(pmxt) = &self.pmxt_pool {
            pmxt.close_all().await;
        }
        Ok(())
    }
}
