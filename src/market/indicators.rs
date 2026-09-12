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

/// Compute the Relative Strength Index (RSI) for a price series.
///
/// Uses the standard Wilder's smoothing method.
/// Returns None if `prices.len() <= period`.
pub fn rsi(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() <= period || period == 0 {
        return None;
    }

    let mut gains = 0.0;
    let mut losses = 0.0;

    // Calculate initial average gain/loss over the first `period` changes
    for i in 1..=period {
        let diff = prices[i] - prices[i - 1];
        if diff >= 0.0 {
            gains += diff;
        } else {
            losses += diff.abs();
        }
    }

    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    // Wilder's smoothing for subsequent periods
    for i in (period + 1)..prices.len() {
        let diff = prices[i] - prices[i - 1];
        let (gain, loss) = if diff >= 0.0 {
            (diff, 0.0)
        } else {
            (0.0, diff.abs())
        };

        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;
    }

    if avg_loss == 0.0 {
        if avg_gain == 0.0 {
            return Some(50.0);
        }
        return Some(100.0);
    }

    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
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
        assert!(e + 1e-6 >= s, "EMA {} should be >= SMA {} on rising series", e, s);
    }

    #[test]
    fn rsi_returns_none_when_insufficient_prices() {
        assert_eq!(rsi(&[10.0, 11.0, 12.0], 14), None);
    }

    #[test]
    fn rsi_all_gains_is_100() {
        let prices: Vec<f64> = (1..=30).map(|i| i as f64 * 2.0).collect();
        let r = rsi(&prices, 14).unwrap();
        assert!(approx_eq(r, 100.0), "Monotonically increasing series should give RSI=100, got {}", r);
    }

    #[test]
    fn rsi_all_losses_is_0() {
        let prices: Vec<f64> = (1..=30).rev().map(|i| i as f64 * 2.0).collect();
        let r = rsi(&prices, 14).unwrap();
        assert!(approx_eq(r, 0.0), "Monotonically decreasing series should give RSI=0, got {}", r);
    }

    #[test]
    fn rsi_flat_prices_is_50() {
        let prices = vec![100.0; 30];
        let r = rsi(&prices, 14).unwrap();
        assert!(approx_eq(r, 50.0), "Flat series should give RSI=50, got {}", r);
    }
}
