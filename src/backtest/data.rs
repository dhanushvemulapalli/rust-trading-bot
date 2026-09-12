//! Historical market data ingestion and synthetic generation.

use crate::market::types::Bar;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Load historical bars from a CSV file.
/// Expected header: symbol,open,high,low,close,volume,timestamp
pub fn load_bars_from_csv(path: impl AsRef<Path>) -> Result<Vec<Bar>> {
    let file = File::open(&path).with_context(|| format!("Failed to open CSV file {:?}", path.as_ref()))?;
    let reader = BufReader::new(file);
    let mut bars = Vec::new();

    for (line_num, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        if line_num == 0 && line.to_lowercase().contains("symbol") {
            continue; // Skip header
        }
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 7 {
            continue;
        }

        let symbol = parts[0].to_string();
        let open = parts[1].parse::<f64>()?;
        let high = parts[2].parse::<f64>()?;
        let low = parts[3].parse::<f64>()?;
        let close = parts[4].parse::<f64>()?;
        let volume = parts[5].parse::<u64>()?;
        let timestamp = parts[6].to_string();

        bars.push(Bar {
            symbol,
            open,
            high,
            low,
            close,
            volume,
            timestamp,
        });
    }

    Ok(bars)
}

/// Generate synthetic price data with a simulated trend and cycle.
pub fn generate_synthetic_bars(symbol: &str, start_price: f64, count: usize) -> Vec<Bar> {
    let mut bars = Vec::with_capacity(count);
    for i in 0..count {
        // Multi-frequency wave to produce realistic trend crossovers
        let wave = (i as f64 * 0.08).sin() * 5.0 + (i as f64 * 0.02).sin() * 15.0;
        let trend = i as f64 * 0.05;
        let price = (start_price + wave + trend).max(1.0);

        let high = price + 1.2;
        let low = price - 1.1;
        let open = price - 0.2;
        let close = price;

        bars.push(Bar {
            symbol: symbol.into(),
            open,
            high,
            low,
            close,
            volume: 100_000,
            timestamp: format!("2024-01-01T{:02}:{:02}:00Z", (i / 60) % 24, i % 60),
        });
    }

    bars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_synthetic_bars() {
        let bars = generate_synthetic_bars("AAPL", 150.0, 100);
        assert_eq!(bars.len(), 100);
        assert_eq!(bars[0].symbol, "AAPL");
        assert!(bars[0].close > 0.0);
    }
}
