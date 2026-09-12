//! SQLite database operations using SQLx.
//!
//! All queries go through `Db`. No other module should hold a reference to the pool.

use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tracing::info;
use uuid::Uuid;

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    /// Connect to (or create) the SQLite database and run migrations.
    pub async fn connect(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    /// Run embedded migrations to create tables if they don't exist.
    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bars (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol      TEXT NOT NULL,
                open        REAL NOT NULL,
                high        REAL NOT NULL,
                low         REAL NOT NULL,
                close       REAL NOT NULL,
                volume      INTEGER NOT NULL,
                timestamp   TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS orders (
                id              TEXT PRIMARY KEY,
                client_order_id TEXT NOT NULL,
                symbol          TEXT NOT NULL,
                side            TEXT NOT NULL,
                qty             REAL NOT NULL,
                filled_qty      REAL NOT NULL DEFAULT 0,
                status          TEXT NOT NULL,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS signals (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                symbol      TEXT NOT NULL,
                signal      TEXT NOT NULL,
                timestamp   TEXT NOT NULL,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
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
        sqlx::query!(
            "INSERT INTO bars (symbol, open, high, low, close, volume, timestamp)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            symbol, open, high, low, close, volume, timestamp
        )
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
        sqlx::query!(
            r#"INSERT INTO orders (id, client_order_id, symbol, side, qty, filled_qty, status)
               VALUES (?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(id) DO UPDATE SET
                   filled_qty = excluded.filled_qty,
                   status = excluded.status,
                   updated_at = datetime('now')"#,
            id_str, client_order_id, symbol, side, qty, filled_qty, status
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Signals ───────────────────────────────────────────────────────────────

    pub async fn insert_signal(&self, symbol: &str, signal: &str, timestamp: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO signals (symbol, signal, timestamp) VALUES (?, ?, ?)",
            symbol, signal, timestamp
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
