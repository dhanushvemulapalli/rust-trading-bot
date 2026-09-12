//! SQLite database operations using SQLx.
//!
//! All queries go through `Db`. No other module should hold a reference to the pool.
//!
//! We use `sqlx::query()` (the non-macro variant) intentionally so that the
//! compiler does not need a live database connection at build time.

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    /// Connect to (or create) the SQLite database and run migrations.
    pub async fn connect(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    /// Create tables if they do not already exist.
    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS bars (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol      TEXT    NOT NULL,
                open        REAL    NOT NULL,
                high        REAL    NOT NULL,
                low         REAL    NOT NULL,
                close       REAL    NOT NULL,
                volume      INTEGER NOT NULL,
                timestamp   TEXT    NOT NULL,
                created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS orders (
                id              TEXT PRIMARY KEY,
                client_order_id TEXT NOT NULL,
                symbol          TEXT NOT NULL,
                side            TEXT NOT NULL,
                qty             REAL NOT NULL,
                filled_qty      REAL NOT NULL DEFAULT 0,
                status          TEXT NOT NULL,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS signals (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol     TEXT NOT NULL,
                signal     TEXT NOT NULL,
                timestamp  TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS positions (
                symbol          TEXT PRIMARY KEY,
                side            TEXT NOT NULL,
                qty             REAL NOT NULL,
                avg_entry_price REAL NOT NULL,
                market_value    REAL NOT NULL,
                unrealized_pl   REAL NOT NULL,
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await?;

        info!("Database migration complete");
        Ok(())
    }

    // ── Bars ──────────────────────────────────────────────────────────────────

    pub async fn insert_bar(
        &self,
        symbol: &str,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: i64,
        timestamp: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO bars (symbol, open, high, low, close, volume, timestamp)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(symbol)
        .bind(open)
        .bind(high)
        .bind(low)
        .bind(close)
        .bind(volume)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Orders ────────────────────────────────────────────────────────────────

    pub async fn upsert_order(
        &self,
        id: Uuid,
        client_order_id: &str,
        symbol: &str,
        side: &str,
        qty: f64,
        filled_qty: f64,
        status: &str,
    ) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query(
            "INSERT INTO orders (id, client_order_id, symbol, side, qty, filled_qty, status)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                 filled_qty = excluded.filled_qty,
                 status     = excluded.status,
                 updated_at = datetime('now')",
        )
        .bind(id_str)
        .bind(client_order_id)
        .bind(symbol)
        .bind(side)
        .bind(qty)
        .bind(filled_qty)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Signals ───────────────────────────────────────────────────────────────

    pub async fn insert_signal(&self, symbol: &str, signal: &str, timestamp: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO signals (symbol, signal, timestamp) VALUES (?, ?, ?)",
        )
        .bind(symbol)
        .bind(signal)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_order_status(&self, id: Uuid, status: &str, filled_qty: f64) -> Result<()> {
        let id_str = id.to_string();
        sqlx::query(
            "UPDATE orders SET status = ?, filled_qty = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(status)
        .bind(filled_qty)
        .bind(id_str)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Positions ─────────────────────────────────────────────────────────────

    pub async fn upsert_position(&self, pos: &crate::market::types::Position) -> Result<()> {
        let side = format!("{:?}", pos.side);
        sqlx::query(
            "INSERT INTO positions (symbol, side, qty, avg_entry_price, market_value, unrealized_pl)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(symbol) DO UPDATE SET
                 side = excluded.side,
                 qty = excluded.qty,
                 avg_entry_price = excluded.avg_entry_price,
                 market_value = excluded.market_value,
                 unrealized_pl = excluded.unrealized_pl,
                 updated_at = datetime('now')",
        )
        .bind(&pos.symbol)
        .bind(side)
        .bind(pos.qty)
        .bind(pos.avg_entry_price)
        .bind(pos.market_value)
        .bind(pos.unrealized_pl)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_position(&self, symbol: &str) -> Result<()> {
        sqlx::query("DELETE FROM positions WHERE symbol = ?")
            .bind(symbol)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
