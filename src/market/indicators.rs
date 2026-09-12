//! Technical indicators used by strategies.
//!
//! All indicators operate on `f64` slices (e.g. a window of closing prices)
//! and are pure functions — no external state required.

/// Compute the Exponential Moving Average for the given price series.
///
/// The EMA uses the standard smoothing factor k = 2 / (period + 1).
/// Returns `None` if `prices` has fewer elements than `period`.
pub fn ema(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }
    let k = 2.0 / (period as f64 + 1.0);
    // Seed with the simple average of the first `period` elements
    let seed: f64 = prices[..period].iter().sum::<f64>() / period as f64;
    let result = prices[period..].iter().fold(seed, |prev, &price| {
        price * k + prev * (1.0 - k)
    });
    Some(result)
}

/// Compute the Simple Moving Average of the last `period` values.
pub fn sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }
    let window = &prices[prices.len() - period..];
    Some(window.iter().sum::<f64>() / period as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn ema_returns_none_when_too_few_prices() {
        assert_eq!(ema(&[100.0, 101.0], 3), None);
    }

    #[test]
    fn ema_of_constant_series_equals_constant() {
        let prices = vec![50.0; 30];
        let result = ema(&prices, 20).unwrap();
        assert!(approx_eq(result, 50.0), "EMA of constant should be constant, got {}", result);
    }

    #[test]
    fn sma_is_correct() {
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert!(approx_eq(sma(&prices, 3).unwrap(), 40.0));
    }

    #[test]
    fn ema_rising_series_is_above_sma() {
        // Rising prices → EMA (more weight on recent) should be >= SMA
        let prices: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        let e = ema(&prices, 10).unwrap();
        let s = sma(&prices, 10).unwrap();
        assert!(e >= s, "EMA {} should be >= SMA {} on rising series", e, s);
    }
}
