# Trading Bot

A Rust-based algorithmic trading bot that connects to Alpaca Paper Trading.

## Architecture

```
Market Data (WebSocket)
    ↓
Strategy (EMA Crossover)
    ↓
Signal (Buy / Sell / Hold)
    ↓
Risk Manager (size limits, daily loss cap, duplicate protection)
    ↓
Approved Order
    ↓
Order Manager
    ↓
Alpaca Paper Trading REST API
    ↓
Trade Updates (WebSocket) ← lifecycle events feed back to Order Manager
```

## Modules

| Module | Responsibility |
|---|---|
| `config` | Load `.env` config, guard against live trading |
| `alpaca/` | All Alpaca-specific HTTP and WebSocket logic |
| `market/` | Domain types (`Bar`, `Trade`, `Quote`, `Position`) and indicators (`ema`, `sma`) |
| `strategy/` | `Strategy` trait + `EmaCrossover` implementation |
| `risk/` | `RiskManager` — approves/rejects signals before execution |
| `execution/` | `OrderManager` (submit, track) + `trade_updates` WebSocket |
| `storage/` | SQLite persistence via SQLx |

## Setup

### 1. Install Rust
```bash
# Windows (PowerShell)
winget install Rustlang.Rustup
# or download from https://rustup.rs
```

### 2. Create `.env`
```
cp .env.example .env
# Edit .env and add your Alpaca paper-trading API key and secret
```

### 3. Build
```bash
cargo build
```

### 4. Run tests
```bash
cargo test
```

### 5. Run the bot
```bash
cargo run
```

## Configuration (`.env`)

| Variable | Default | Description |
|---|---|---|
| `ALPACA_KEY` | *(required)* | Alpaca paper API key ID |
| `ALPACA_SECRET` | *(required)* | Alpaca paper API secret key |
| `ALPACA_BASE_URL` | `https://paper-api.alpaca.markets` | REST base URL |
| `ALPACA_DATA_WS` | `wss://stream.data.alpaca.markets/v2/iex` | Market data WS |
| `ALPACA_TRADING_WS` | `wss://paper-api.alpaca.markets/stream` | Trade updates WS |
| `USE_PAPER` | `true` | Must be `true`. Live trading is not enabled. |
| `DATABASE_URL` | `sqlite://./trading_bot.db` | SQLite database path |

## Security

- **Never commit `.env`** — it is in `.gitignore`.
- Paper and live credentials must never be mixed.
- The bot exits immediately if `USE_PAPER=false`.

## Current Strategy: EMA Crossover

| Parameter | Value |
|---|---|
| Short EMA period | 20 bars |
| Long EMA period | 50 bars |
| Buy signal | EMA(20) crosses above EMA(50) |
| Sell signal | EMA(20) crosses below EMA(50) |

> ⚠️ This strategy is a proof-of-concept to validate the architecture. It is not tested for profitability.

## Risk Management

Current limits (all configurable via `RiskConfig`):

| Check | Default |
|---|---|
| Max position size | 10 shares |
| Max open positions | 5 |
| Daily loss limit | -$500 |
| Duplicate order protection | Enabled |

## Roadmap

- [x] Milestone 1 – Alpaca connectivity
- [x] Milestone 2 – Market data (WebSocket)
- [x] Milestone 3 – Domain market types
- [x] Milestone 4 – EMA strategy
- [x] Milestone 5 – Risk manager
- [x] Milestone 6 – Paper order submission
- [x] Milestone 7 – Full persistence (positions, orders, signals, bars)
- [x] Milestone 8 – Backtesting engine (`cargo run --example run_backtest`)
- [ ] Milestone 9 – Monitoring / metrics
- [ ] Milestone 10 – Docker + VPS deployment
