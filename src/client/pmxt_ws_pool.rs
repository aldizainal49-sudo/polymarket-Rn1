use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use dashmap::DashMap;
use tracing::{info, error};
use anyhow::Result;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct PmxtPriceUpdate {
    pub venue: String,
    pub market_id: String,
    pub token_id: String,
    pub price: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source_connection: u32,
}

#[derive(Debug)]
struct PmxtConnection {
    id: u32,
    venue: String,
    write: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
}

#[derive(Debug)]
pub struct PmxtWebSocketPool {
    connections: Arc<DashMap<u32, Arc<Mutex<PmxtConnection>>>>,
    price_cache: Arc<DashMap<String, PmxtPriceUpdate>>,
    num_connections: u32,
    venues: Vec<String>,
}

impl PmxtWebSocketPool {
    pub async fn new(num_connections: u32, venues: Vec<String>) -> Result<Self> {
        let venues_clone = venues.clone();
        let pool = Self {
            connections: Arc::new(DashMap::new()),
            price_cache: Arc::new(DashMap::new()),
            num_connections,
            venues: venues_clone,
        };

        for i in 0..num_connections {
            let venue = venues[i as usize % venues.len()].clone();
            match pool.connect_websocket(i, &venue).await {
                Ok(conn) => {
                    pool.connections.insert(i, Arc::new(Mutex::new(conn)));
                    info!("PMXT WebSocket #{} connected to {}", i, venue);
                }
                Err(e) => {
                    error!("PMXT WebSocket #{} failed: {}", i, e);
                }
            }
        }
        Ok(pool)
    }

    async fn connect_websocket(&self, id: u32, venue: &str) -> Result<PmxtConnection> {
        let ws_url = match venue {
            "polymarket" => "wss://ws-subscriptions-clob.polymarket.com/ws/market",
            "kalshi" => "wss://trading-api.kalshi.com/trade/ws",
            "smarkets" => "wss://ws-api.smarkets.com/v2/",
            _ => "wss://api.pmxt.dev/ws",
        };

        let (ws_stream, _) = connect_async(ws_url).await?;
        Ok(PmxtConnection { id, venue: venue.to_string(), write: ws_stream })
    }

    pub async fn subscribe_market(&self, market_id: &str, token_ids: Vec<String>) -> Result<()> {
        let subscribe_msg = json!({ "type": "subscribe", "market_id": market_id, "token_ids": token_ids }).to_string();
        for entry in self.connections.iter() {
            let conn = entry.value();
            let mut guard = conn.lock().await;
            let _ = guard.write.send(Message::Text(subscribe_msg.clone())).await;
        }
        Ok(())
    }

    pub async fn run(&self, tx: tokio::sync::mpsc::Sender<PmxtPriceUpdate>) {
        let mut handles = Vec::new();
        for entry in self.connections.iter() {
            let id = *entry.key();
            let conn = entry.value().clone();
            let tx_clone = tx.clone();
            let price_cache = self.price_cache.clone();

            let handle = tokio::spawn(async move {
                let mut guard = conn.lock().await;
                while let Some(msg) = guard.write.next().await {
                    if let Ok(Message::Text(text)) = msg {
                        if let Ok(data) = serde_json::from_str::<Value>(&text) {
                            if let Some(price
