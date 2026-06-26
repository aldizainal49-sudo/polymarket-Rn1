mod config;
mod client;
mod strategy;
mod order;
mod ai_client;
mod logger;
mod backtest;

use std::env;
use dotenv::dotenv;
use tracing_subscriber;
use tracing::info;
use anyhow::Result;
use clap::Parser;

use config::Config;
use strategy::engine::TradingEngine;
use ai_client::AIClient;
use logger::SheetLogger;
use client::sportmonks::SportmonksClient;
use client::sportsdataio::SportsDataIOClient;
use client::sportradar::SportradarClient;
use client::pmxt_ws_pool::create_pmxt_pool;

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value_t = false)]
    paper: bool,

    #[arg(short, long, default_value = "config.json")]
    config: String,

    #[arg(long, default_value_t = false)]
    backtest: bool,

    #[arg(long, default_value = "data/historical.csv")]
    backtest_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    dotenv().ok();

    let cli = Cli::parse();

    // === BACKTEST MODE ===
    if cli.backtest {
        info!("📊 Running Backtest Mode...");
        let config = Config::from_file(&cli.config)?;
        let mut backtest_engine = backtest::BacktestEngine::new(
            config.trading.clone(),
            config.trading.paper_balance,
        );
        backtest_engine.run(&cli.backtest_file).await?;
        return Ok(());
    }

    info!("📄 Loading config from: {}", cli.config);
    let mut config = Config::from_file(&cli.config)?;

    if cli.paper {
        config.trading.paper_mode = true;
        info!("📝 PAPER MODE ACTIVATED");
        info!("💰 Starting Paper Balance: ${:.2}", config.trading.paper_balance);
    } else {
        config.trading.paper_mode = false;
        info!("🔴 LIVE MODE ACTIVATED");
    }

    // === AI CLIENT ===
    let ai_client = if let Ok(key) = env::var("GLM_API_KEY") {
        let base_url = env::var("GLM_BASE_URL")
            .unwrap_or_else(|_| "https://api.z.ai/api/v1".to_string());
        let model = env::var("GLM_MODEL")
            .unwrap_or_else(|_| "glm-5.2".to_string());
        info!("🧠 GLM-5.2 AI Client initialized");
        Some(AIClient::new(key, base_url, model))
    } else {
        info!("⚠️ GLM_API_KEY not set. AI prediction disabled.");
        None
    };

    // === GOOGLE SHEETS LOGGER ===
    let logger = if let Ok(creds_path) = env::var("GOOGLE_CREDENTIALS") {
        if let Ok(sheet_id) = env::var("GOOGLE_SHEET_ID") {
            match SheetLogger::new(&creds_path, &sheet_id).await {
                Ok(logger) => {
                    info!("✅ Google Sheets Logger initialized.");
                    Some(logger)
                }
                Err(e) => {
                    info!("⚠️ Failed to initialize Google Sheets Logger: {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // === SPORTMONKS ===
    let sportmonks_client = if let Ok(token) = env::var("SPORTMONKS_API_TOKEN") {
        info!("⚽ Sportmonks client initialized");
        Some(SportmonksClient::new(token))
    } else {
        info!("⚠️ SPORTMONKS_API_TOKEN not set.");
        None
    };

    // === SPORTSDATAIO ===
    let sportsdataio_client = if let Ok(key) = env::var("SPORTSDATAIO_API_KEY") {
        info!("🏀 SportsDataIO client initialized");
        Some(SportsDataIOClient::new(key))
    } else {
        info!("⚠️ SPORTSDATAIO_API_KEY not set.");
        None
    };

    // === SPORTRADAR ===
    let sportradar_client = if let Ok(key) = env::var("SPORTRADAR_API_KEY") {
        info!("🏟️ Sportradar client initialized");
        Some(SportradarClient::new(key))
    } else {
        info!("⚠️ SPORTRADAR_API_KEY not set.");
        None
    };

    // === PMXT WEB SOCKET POOL (5 connections) ===
    let pmxt_pool = match create_pmxt_pool().await {
        Ok(pool) => {
            info!("✅ PMXT WebSocket Pool with 5 connections created");
            Some(pool)
        }
        Err(e) => {
            info!("⚠️ PMXT WebSocket Pool failed: {}", e);
            None
        }
    };

    let mut engine = TradingEngine::new(
        config,
        ai_client,
        logger,
        sportmonks_client,
        sportsdataio_client,
        sportradar_client,
        pmxt_pool,
    ).await?;

    let engine_clone = engine;
    ctrlc::set_handler(move || {
        info!("⚠️ Received shutdown signal");
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                if let Err(e) = engine_clone.shutdown().await {
                    eprintln!("Shutdown error: {}", e);
                }
            });
        std::process::exit(0);
    })?;

    engine.run().await?;

    Ok(())
}
