use std::collections::VecDeque;
use reqwest::Client;
use rust_decimal::Decimal;
use serde_json::json;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;
use chrono::Utc;

use crate::config::Config;
use crate::client::types::{Order, OrderSide, OrderStatus, OrderBook};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug)]
pub struct ClobClient {
    client: Client,
    base_url: String,
    api_key: String,
    api_secret: String,
    passphrase: String,
}