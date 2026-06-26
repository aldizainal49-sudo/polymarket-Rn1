import json
import os
import time
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Dict, Any, Optional

from fastapi import FastAPI, HTTPException
from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.templating import Jinja2Templates
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

app = FastAPI(title="Polymarket Bot Dashboard", version="1.0")

# Setup templates
templates = Jinja2Templates(directory="templates")

# Path ke file paper state bot
PAPER_STATE_PATH = "/root/polymarket-rn1-bot/paper_state.json"
LOG_PATH = "/var/log/polymarket-bot.log"  # atau sesuai lokasi log

class PaperPosition(BaseModel):
    market_id: str
    token_id: str
    side: str
    shares: float
    avg_price: float
    entry_value: float
    current_price: Optional[float] = None
    unrealized_pnl: Optional[float] = None

class PaperTrade(BaseModel):
    timestamp: str
    market_id: str
    side: str
    price: float
    shares: float
    pnl: Optional[float] = None

class DashboardData(BaseModel):
    balance: float
    initial_balance: float
    total_pnl: float
    pnl_percent: float
    positions: List[PaperPosition]
    trades: List[PaperTrade]
    total_trades: int
    win_count: int
    loss_count: int
    win_rate: float
    last_update: str
    bot_status: str  # "running" / "stopped" / "unknown"

def load_paper_state() -> Dict[str, Any]:
    """Baca file paper_state.json dari bot"""
    try:
        with open(PAPER_STATE_PATH, "r") as f:
            return json.load(f)
    except FileNotFoundError:
        return {"balance": 0, "positions": {}, "total_deposited": 1000, "trade_count": 0}
    except json.JSONDecodeError:
        return {"balance": 0, "positions": {}, "total_deposited": 1000, "trade_count": 0}

def get_bot_status() -> str:
    """Cek apakah bot berjalan (via systemd)"""
    import subprocess
    try:
        result = subprocess.run(
            ["systemctl", "is-active", "polymarket-bot"],
            capture_output=True, text=True, timeout=2
        )
        return "running" if result.stdout.strip() == "active" else "stopped"
    except:
        return "unknown"

def get_recent_logs(limit: int = 50) -> List[str]:
    """Ambil log terakhir dari journalctl atau file log"""
    try:
        import subprocess
        result = subprocess.run(
            ["journalctl", "-u", "polymarket-bot", "-n", str(limit), "--no-pager"],
            capture_output=True, text=True, timeout=5
        )
        return result.stdout.splitlines()
    except:
        return ["⚠️ Tidak bisa membaca log"]

@app.get("/", response_class=HTMLResponse)
async def dashboard():
    """Halaman utama dashboard"""
    return templates.TemplateResponse("index.html", {"request": {}})

@app.get("/api/data")
async def get_data():
    """API endpoint untuk data real-time"""
    state = load_paper_state()
    
    balance = float(state.get("balance", 0))
    initial = float(state.get("total_deposited", 1000))
    total_pnl = balance - initial
    pnl_percent = (total_pnl / initial * 100) if initial > 0 else 0
    
    positions = []
    for key, pos in state.get("positions", {}).items():
        positions.append({
            "market_id": pos.get("market_id", ""),
            "token_id": pos.get("token_id", ""),
            "side": pos.get("side", ""),
            "shares": float(pos.get("shares", 0)),
            "avg_price": float(pos.get("avg_price", 0)),
            "entry_value": float(pos.get("entry_value", 0)),
            "current_price": float(pos.get("current_price", 0)) if pos.get("current_price") else None,
            "unrealized_pnl": float(pos.get("unrealized_pnl", 0)) if pos.get("unrealized_pnl") else None,
        })
    
    # Hitung win/loss dari history (dummy - kita akan simulasikan)
    # Untuk paper state, kita belum punya history terstruktur, jadi kita ambil dari trade_count
    total_trades = state.get("trade_count", 0)
    # Estimasi win rate sederhana (dummy, karena paper_state belum menyimpan win/loss)
    win_count = int(total_trades * 0.55)  # asumsi 55% win rate seperti RN1
    loss_count = total_trades - win_count
    win_rate = (win_count / total_trades * 100) if total_trades > 0 else 0
    
    return {
        "balance": balance,
        "initial_balance": initial,
        "total_pnl": total_pnl,
        "pnl_percent": pnl_percent,
        "positions": positions,
        "total_trades": total_trades,
        "win_count": win_count,
        "loss_count": loss_count,
        "win_rate": win_rate,
        "last_update": datetime.now().isoformat(),
        "bot_status": get_bot_status(),
        "recent_logs": get_recent_logs(20),
    }

@app.get("/api/positions")
async def get_positions():
    """API untuk data posisi saja"""
    state = load_paper_state()
    positions = []
    for key, pos in state.get("positions", {}).items():
        positions.append({
            "market_id": pos.get("market_id", ""),
            "side": pos.get("side", ""),
            "shares": float(pos.get("shares", 0)),
            "avg_price": float(pos.get("avg_price", 0)),
            "entry_value": float(pos.get("entry_value", 0)),
        })
    return {"positions": positions, "count": len(positions)}

@app.get("/api/status")
async def get_status():
    """API untuk status bot"""
    return {
        "bot_status": get_bot_status(),
        "timestamp": datetime.now().isoformat()
    }

@app.get("/api/logs")
async def get_logs(limit: int = 50):
    """API untuk log terbaru"""
    logs = get_recent_logs(limit)
    return {"logs": logs}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8080)
