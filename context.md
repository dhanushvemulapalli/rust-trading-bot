# Trading Bot — Project Context

## Project Overview

We are building a trading bot using:

* **Rust** as the primary programming language
* **Alpaca API** as the broker/trading API
* **VS Code** as the IDE
* **Ollama** for local AI-assisted coding
* **Continue** (or another VS Code agent extension) as the bridge between VS Code and Ollama

The goal is to build this as a serious engineering project rather than a simple trading script.

---

## Current Objective

Build an event-driven trading system capable of:

1. Receiving market data from Alpaca.
2. Processing market data in Rust.
3. Running a trading strategy.
4. Generating buy/sell signals.
5. Applying risk-management rules.
6. Creating and managing orders through Alpaca.
7. Tracking order/fill/position state.
8. Persisting relevant trading data.
9. Supporting backtesting.
10. Running against Alpaca Paper Trading before any consideration of live trading.

The initial objective is **not immediate profitability**.

The first milestone is a reliable end-to-end paper-trading system.

---

## High-Level Architecture

```text
                    ┌─────────────────────┐
                    │     Alpaca API      │
                    │                     │
                    │ Trading REST API    │
                    │ Market Data WS      │
                    │ Trade Updates WS    │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   Market Data       │
                    │     Collector       │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │      Strategy       │
                    │                     │
                    │ Indicators          │
                    │ Signals             │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │    Risk Manager     │
                    │                     │
                    │ Position sizing     │
                    │ Exposure limits     │
                    │ Stop loss           │
                    │ Max loss            │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │   Order Manager     │
                    │                     │
                    │ Submit              │
                    │ Cancel              │
                    │ Replace             │
                    │ Track               │
                    └──────────┬──────────┘
                               │
                               ▼
                         Alpaca Trading
```

---

## Important Architectural Principle

The strategy must **not directly interact with Alpaca**.

Use this flow:

```text
Market Data
    ↓
Strategy
    ↓
Signal
    ↓
Risk Manager
    ↓
Approved Order
    ↓
Execution / Order Manager
    ↓
Alpaca
```

This separation is important for testing, backtesting, and eventually changing brokers.

The same strategy and risk-management code should ideally work with:

* Historical data during backtesting
* Simulated data during tests
* Live Alpaca market data

---

## Proposed Rust Structure

```text
trading-bot/
│
├── Cargo.toml
├── Cargo.lock
├── .env
├── .gitignore
├── context.md
├── README.md
│
├── src/
│   ├── main.rs
│   ├── config.rs
│   │
│   ├── alpaca/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   ├── market_data.rs
│   │   ├── orders.rs
│   │   └── account.rs
│   │
│   ├── market/
│   │   ├── mod.rs
│   │   ├── candle.rs
│   │   ├── tick.rs
│   │   └── indicators.rs
│   │
│   ├── strategy/
│   │   ├── mod.rs
│   │   └── ...
│   │
│   ├── risk/
│   │   ├── mod.rs
│   │   └── manager.rs
│   │
│   ├── execution/
│   │   ├── mod.rs
│   │   └── order_manager.rs
│   │
│   └── storage/
│       ├── mod.rs
│       └── database.rs
│
└── tests/
```

This is a proposed structure, not a requirement. Keep the project simple until complexity is justified.

---

## Rust Technology Stack

Initial dependencies to consider:

```text
tokio
reqwest
tokio-tungstenite
serde
serde_json
dotenvy
tracing
tracing-subscriber
anyhow
thiserror
chrono
sqlx
uuid
```

### Purpose

| Library             | Purpose                       |
| ------------------- | ----------------------------- |
| `tokio`             | Async runtime                 |
| `reqwest`           | HTTP/REST requests            |
| `tokio-tungstenite` | WebSocket connections         |
| `serde`             | Serialization/deserialization |
| `serde_json`        | JSON                          |
| `dotenvy`           | Environment configuration     |
| `tracing`           | Structured logging            |
| `anyhow`            | Application-level errors      |
| `thiserror`         | Typed domain errors           |
| `chrono`            | Date/time handling            |
| `sqlx`              | Database access               |
| `uuid`              | IDs                           |

Do not add libraries merely because they are available. Prefer a small dependency footprint.

---

## Alpaca Integration

Use **Alpaca Paper Trading** during development.

Required Alpaca functionality:

### REST API

* Account information
* Positions
* Orders
* Order status
* Submit order
* Cancel order
* Replace order where required

### Market Data WebSocket

Potential data:

* Trades
* Quotes
* Bars

### Trading WebSocket

Track:

* Order submitted
* Order accepted
* Order partially filled
* Order filled
* Order canceled
* Order rejected

The execution layer must account for asynchronous order updates.

---

## API Client Design

Prefer a thin, explicit Alpaca client rather than tightly coupling the entire application to a community Rust SDK.

For example:

```rust
struct AlpacaClient {
    http_client: reqwest::Client,
    base_url: String,
    api_key: String,
    secret_key: String,
}
```

The exact implementation can change.

Keep Alpaca-specific types and logic inside the `alpaca` module as much as possible.

The rest of the application should operate using domain-level types.

---

## Strategy Interface

Strategies should be interchangeable.

A possible abstraction:

```rust
trait Strategy {
    fn on_bar(&mut self, bar: &Bar) -> Signal;
}
```

Possible signals:

```rust
enum Signal {
    Buy,
    Sell,
    Hold,
}
```

The exact interface should be designed properly once the market-data and position model are established.

Do not prematurely over-engineer the strategy abstraction.

---

## Risk Management

Never allow a raw strategy signal to directly create an order.

Example:

```text
Strategy → BUY
             ↓
       Risk Manager
             ↓
      position size
      exposure check
      loss limits
      trading limits
             ↓
        Approved Order
```

Risk management should eventually support concepts such as:

* Maximum position size
* Maximum portfolio exposure
* Maximum loss per trade
* Maximum daily loss
* Stop loss
* Take profit
* Maximum number of open positions
* Duplicate-order protection
* Market/session restrictions

Exact values should be configurable rather than hard-coded.

---

## Order Management

The order manager is responsible for execution state.

It should handle:

```text
Signal
  ↓
Order request
  ↓
Submitted
  ↓
Accepted
  ↓
Partially Filled
  ↓
Filled
```

and failure paths:

```text
Rejected
Canceled
Expired
Failed
```

Do not assume that submitting an order means it was filled.

Use Alpaca trade updates to maintain authoritative execution state.

---

## Persistence

Eventually persist:

* Market data required for backtesting
* Signals
* Orders
* Order status changes
* Fills
* Positions
* P&L
* Errors
* Strategy decisions
* Relevant timestamps

PostgreSQL is the preferred production database.

SQLite can be used during early development if it substantially simplifies local development.

---

## Backtesting

Backtesting is an important part of the architecture.

The strategy should not depend directly on live Alpaca connections.

Desired design:

```text
                 ┌──────────────────┐
                 │     Strategy     │
                 └────────┬─────────┘
                          │
             ┌────────────┴────────────┐
             ↓                         ↓
      Historical Data             Live Data
             ↓                         ↓
       Backtest Engine             Alpaca
```

The goal is to reuse strategy logic between backtesting and live/paper execution.

A backtest should account for realistic assumptions such as:

* Fees
* Slippage
* Spread
* Execution latency where relevant
* Position sizing
* Order types
* Partial fills where modeled
* Market hours

Do not judge a strategy based only on raw historical price movement.

---

## Initial Development Milestones

### Milestone 1 — Connectivity

Build:

```text
Rust
 ↓
Alpaca Paper API
 ↓
GET account
 ↓
Display account information
```

Verify authentication and configuration.

### Milestone 2 — Market Data

Connect to Alpaca WebSocket.

Receive and deserialize live market data.

Log:

```text
timestamp
symbol
price
volume
```

### Milestone 3 — Market Model

Create internal types for:

* Tick
* Quote
* Trade
* Candle/Bar
* Position
* Order

### Milestone 4 — Strategy

Implement one extremely simple strategy.

Example:

```text
EMA 20
EMA 50
    ↓
20 crosses above 50 → BUY
20 crosses below 50 → SELL
```

The first strategy exists primarily to validate the architecture.

### Milestone 5 — Risk

Insert the risk manager between strategy and execution.

### Milestone 6 — Paper Execution

Allow the bot to submit paper orders.

Track their lifecycle using trade updates.

### Milestone 7 — Persistence

Store trades, orders, fills and relevant market data.

### Milestone 8 — Backtesting

Build a historical-data execution path.

### Milestone 9 — Monitoring

Add useful metrics and logs.

### Milestone 10 — Deployment

Only after extensive paper testing:

```text
GitHub
 ↓
Docker
 ↓
Linux VPS
 ↓
24/7 bot
```

Live trading should be treated as a separate stage, not an automatic consequence of successful paper testing.

---

# Development Environment

## IDE

Use:

**VS Code**

Recommended extensions:

* `rust-analyzer`
* `CodeLLDB`
* TOML support
* Error Lens (optional)

---

# AI-Assisted Coding

The goal is to use AI as an engineering agent, not merely an autocomplete engine.

Preferred zero-cost/local architecture:

```text
VS Code
   ↓
Continue / compatible coding agent
   ↓
Ollama
   ↓
Local coding model
```

Ollama provides local model inference.

The VS Code agent layer provides capabilities such as:

* Reading repository files
* Creating files
* Editing files
* Running commands
* Running tests
* Inspecting compiler errors
* Iterating on implementations

The AI must not be blindly trusted with trading logic.

All AI-generated trading code must be reviewed and tested.

---

## AI Coding Rules

When modifying the repository:

1. Inspect existing code before making changes.
2. Do not rewrite unrelated files.
3. Preserve the existing architecture unless there is a clear reason to change it.
4. Run `cargo check` after Rust changes.
5. Run `cargo test` when tests exist.
6. Prefer small, reviewable changes.
7. Do not hard-code credentials.
8. Do not introduce live trading accidentally.
9. Keep paper-trading configuration clearly separated from live configuration.
10. Never assume an order was filled merely because an API request succeeded.
11. Add tests for important trading and risk-management logic.
12. Explain significant architectural changes before implementing them when the change affects multiple modules.

---

# Security

Never commit:

```text
.env
API keys
Secret keys
Access tokens
Passwords
Private credentials
```

`.gitignore` should include:

```text
.env
target/
*.log
```

Use environment variables for credentials.

Paper and live credentials should never be mixed.

The default configuration should be **paper trading**, not live trading.

---

# Engineering Principles

## 1. Correctness before profitability

The initial goal is to build a reliable trading system.

Do not optimize for profits before the execution pipeline is trustworthy.

## 2. Separation of concerns

Keep:

```text
Market Data
Strategy
Risk
Execution
Persistence
```

separate.

## 3. Testability

Important components should be testable without connecting to Alpaca.

## 4. Observability

Every important decision should be explainable from logs/state.

For example:

```text
10:30:01
AAPL
EMA20 crossed EMA50
Signal = BUY
Risk check = PASS
Position size = 10
Order submitted
Order ID = ...
Order filled
Fill price = ...
```

## 5. Fail safely

When something unexpected happens:

```text
unknown state
API failure
WebSocket disconnect
database failure
invalid market data
unexpected order state
```

the bot should fail closed rather than blindly placing additional orders.

## 6. Avoid premature complexity

Start with one strategy, one market/data type, paper trading, and a simple persistence layer.

Add complexity only when there is a demonstrated need.

---

# Current Technology Direction

Current preferred stack:

```text
Language:
Rust

IDE:
VS Code

Rust tooling:
rust-analyzer + CodeLLDB

Broker:
Alpaca

AI:
Ollama + VS Code agent/Continue

HTTP:
reqwest

WebSocket:
tokio-tungstenite

Async:
Tokio

Serialization:
Serde

Database:
PostgreSQL

Observability:
tracing

Deployment:
Docker + Linux VPS

Version control:
Git + GitHub
```

---

# AI Agent Instruction

You are assisting with the development of this trading bot.

Before implementing anything:

1. Understand the existing repository.
2. Follow the architecture described in this file.
3. Prefer simple, idiomatic Rust.
4. Keep broker-specific logic isolated.
5. Keep strategy logic independent of Alpaca.
6. Keep risk management independent of execution.
7. Default to paper trading.
8. Never introduce live trading behavior unless explicitly requested.
9. Never expose or generate real API credentials.
10. Validate changes with Rust tooling.
11. Add or update tests for meaningful behavior.
12. Do not claim that trading logic is profitable or safe merely because it compiles or passes unit tests.
13. Treat financial execution as safety-critical software: unexpected states should result in no new trades rather than speculative actions.

The project should evolve incrementally from:

```text
Alpaca connectivity
        ↓
Market data
        ↓
Strategy
        ↓
Risk
        ↓
Paper execution
        ↓
Persistence
        ↓
Backtesting
        ↓
Monitoring
        ↓
Deployment
```

Do not skip directly to live trading.
