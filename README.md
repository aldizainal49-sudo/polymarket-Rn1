# Polymarket RN1 Trading Bot v2.0

## Deskripsi

Polymarket RN1 Bot adalah trading bot canggih untuk platform Polymarket yang mendukung multiple trading strategies termasuk:
- **Mispricing Detection** - Mendeteksi peluang arbitrase dan mispricing
- **Market Making** - Menyediakan likuiditas dengan spread yang terkontrol
- **HFT (High Frequency Trading)** - Trading cepat dengan latensi rendah
- **Hedging** - Manajemen risiko dengan korelasi pasar
- **Hold/Zombie Strategy** - Menahan posisi hingga settlement
- **Farming** - Akumulasi posisi pada harga yang menguntungkan

## Fitur

- ✅ Paper Trading Mode (tanpa risiko)
- ✅ Real Trading Mode (dengan API key Polymarket)
- ✅ Backtesting dengan data historis
- ✅ AI Prediction (GLM-5.2)
- ✅ Multi Sports Data Provider (Sportmonks, SportsDataIO, Sportradar)
- ✅ WebSocket real-time untuk price updates
- ✅ Google Sheets logging (opsional)
- ✅ PMXT WebSocket Pool (5 connections)

## Persyaratan Sistem

- Rust 1.70+
- Tokio Runtime
- Internet Connection

## Instalasi

### 1. Clone Repository
```bash
git clone https://github.com/zainalaldi150-beep/polymarket-rn1-bot.git
cd polymarket-rn1-bot
```

### 2. Build Project
```bash
# Development build
cargo build

# Release build (rekomendasi untuk production)
cargo build --release
```

Binary akan berada di `target/release/polymarket-rn1-bot`

### 3. Setup Environment Variables

Buat file `.env` di root directory:
```bash
# Polymarket API
POLYMARKET_API_KEY=your_api_key
POLYMARKET_API_SECRET=your_api_secret
POLYMARKET_API_PASSPHRASE=your_passphrase
POLYMARKET_PRIVATE_KEY=your_private_key

# AI (Optional - GLM-5.2)
GLM_API_KEY=your_glm_api_key
GLM_BASE_URL=https://api.z.ai/api/v1
GLM_MODEL=glm-5.2

# Google Sheets (Optional)
GOOGLE_CREDENTIALS=path/to/credentials.json
GOOGLE_SHEET_ID=your_sheet_id

# Sports Data (Optional)
SPORTMONKS_API_TOKEN=your_token
SPORTSDATAIO_API_KEY=your_key
SPORTRADAR_API_KEY=your_key
```

### 4. Konfigurasi

Edit file `config.json`:
```json
{
  "polymarket": {
    "clob_api_url": "https://clob.polymarket.com/api",
    "gamma_api_url": "https://gamma-api.polymarket.com",
    "ws_url": "wss://ws-subscriptions-clob.polymarket.com/ws/market",
    "api_key": "",
    "api_secret": "",
    "api_passphrase": "",
    "private_key": ""
  },
  "trading": {
    "max_markets": 10,
    "scan_interval_ms": 1000,
    "min_liquidity": 1000,
    "mispricing_low_threshold": 0.05,
    "mispricing_high_threshold": 0.15,
    "min_ev_threshold": 0.5,
    "maker_spread": 0.02,
    "max_orders_per_market": 5,
    "order_size": 10,
    "hold_to_settlement": true,
    "max_active_positions": 20,
    "paper_mode": true,
    "paper_balance": 10000
  },
  "hft": {
    "enabled": false,
    "arbitrage_threshold": 0.01,
    "latency_ms": 10,
    "max_arb_size": 50
  },
  "hedging": {
    "enabled": false,
    "max_correlation": 0.8,
    "hedge_ratio": 0.5
  },
  "zombie": {
    "enabled": false,
    "profit_take_threshold": 0.1,
    "ignore_losses": true
  },
  "farming": {
    "enabled": false,
    "min_price": 0.1,
    "max_price": 0.9,
    "farm_size": 100,
    "expiry_window_minutes": 60
  },
  "risk": {
    "max_position_value": 1000,
    "max_drawdown": 0.1,
    "max_order_size": 100,
    "min_price": 0.01,
    "max_price": 0.99
  }
}
```

## Usage

### Paper Trading Mode (Rekomendasi untuk testing)
```bash
./target/release/polymarket-rn1-bot --paper --config config.json
```

### Live Trading Mode
```bash
./target/release/polymarket-rn1-bot --config config.json
```

### Backtesting
```bash
./target/release/polymarket-rn1-bot --backtest --config config.json --backtest-file data/historical.csv
```

### CLI Options
```
--paper              Aktifkan paper trading mode
--config <file>      Path ke file konfigurasi (default: config.json)
--backtest           Jalankan mode backtest
--backtest-file <file>  Path ke file data historis
```

## Deploy ke VPS

### 1. Upload Binary
```bash
# Dari local machine
scp target/release/polymarket-rn1-bot user@your-vps:/opt/polymarket-bot/
scp config.json user@your-vps:/opt/polymarket-bot/
scp .env user@your-vps:/opt/polymarket-bot/
```

### 2. Buat Systemd Service

Buat file `/etc/systemd/system/polymarket-bot.service`:
```ini
[Unit]
Description=Polymarket RN1 Trading Bot
After=network.target

[Service]
User=user
WorkingDirectory=/opt/polymarket-bot
EnvironmentFile=/opt/polymarket-bot/.env
ExecStart=/opt/polymarket-bot/polymarket-rn1-bot --config /opt/polymarket-bot/config.json
Restart=always
RestartSec=30

[Install]
WantedBy=multi-user.target
```

### 3. Aktifkan Service
```bash
sudo systemctl daemon-reload
sudo systemctl enable polymarket-bot
sudo systemctl start polymarket-bot
sudo systemctl status polymarket-bot
```

### 4. Logs
```bash
# Lihat logs
journalctl -u polymarket-bot -f

# Atau
cd /opt/polymarket-bot
tail -f logs/trades_*.csv
```

## Monitoring

Bot akan:
- Menghubungkan ke Polymarket WebSocket
- Scan pasar aktif setiap `scan_interval_ms` milidetik
- Menjalankan semua strategi yang diaktifkan
- Logging semua trade ke file CSV

## Troubleshooting

### Error: "POLY_API_KEY not set"
- Pastikan environment variables sudah di-set
- Pastikan file .env sudah di-load

### Error: "Connection refused"
- Periksa koneksi internet
- Pastikan Polymarket API accessible

### Error: "No markets found"
- Periksa konfigurasi `min_liquidity`
- Pastikan ada pasar aktif di Polymarket

## Kontribusi

1. Fork repository
2. Buat branch feature (`git checkout -b feature/your-feature`)
3. Commit perubahan (`git commit -am 'Add some feature'`)
4. Push ke branch (`git push origin feature/your-feature`)
5. Buat Pull Request

## Lisensi

MIT License

## Kontak

- GitHub: [zainalaldi150-beep](https://github.com/zainalaldi150-beep)
- Project: [polymarket-rn1-bot](https://github.com/zainalaldi150-beep/polymarket-rn1-bot)

---

**Note**: Bot ini untuk tujuan edukasi dan penelitian. Gunakan dengan risiko Anda sendiri. Trading cryptocurrency memiliki risiko tinggi.
