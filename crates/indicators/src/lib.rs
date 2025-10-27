#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
#![deny(missing_docs, unused_must_use)]

//! Pure indicator implementations. Functions are deterministic and return Results on invalid input.
//!
//! Contract: identical input slice -> identical output Vec.

/// Error type for indicators
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum IndicatorError {
    /// Provided period is zero or larger than input length
    #[error("invalid period")]
    InvalidPeriod,
}

/// Simple moving average (SMA).
///
/// Inputs:
/// - `values`: slice of f64 price values
/// - `period`: window size (> 0)
///
/// Returns a Vec<f64> of length `values.len().saturating_sub(period-1)` where index 0 is the first full-window SMA.
pub fn sma(values: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    if period == 0 || period > values.len() {
        return Err(IndicatorError::InvalidPeriod);
    }
    let mut res = Vec::with_capacity(values.len() - period + 1);
    // use iterator windows to avoid direct indexing/slicing
    for window in values.windows(period) {
        let sum = window.iter().copied().sum::<f64>();
        res.push(sum / period as f64);
    }
    Ok(res)
}

/// Exponential moving average (EMA).
///
/// Uses the standard smoothing alpha = 2/(period+1). The first EMA value is the SMA of the first `period` points.
pub fn ema(values: &[f64], period: usize) -> Result<Vec<f64>, IndicatorError> {
    if period == 0 || period > values.len() {
        return Err(IndicatorError::InvalidPeriod);
    }
    let mut res = Vec::with_capacity(values.len() - period + 1);
    // first value: SMA
    let first_sma = values.iter().take(period).copied().sum::<f64>() / period as f64;
    let alpha = 2.0 / (period as f64 + 1.0);
    let mut prev = first_sma;
    res.push(prev);
    for v in values.iter().skip(period).copied() {
        prev = alpha * v + (1.0 - alpha) * prev;
        res.push(prev);
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sma_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        // windows: [1,2,3]=2.0; [2,3,4]=3.0; [3,4,5]=4.0
        assert_eq!(sma(&values, 3), Ok(vec![2.0, 3.0, 4.0]));
    }

    #[test]
    fn ema_basic_matches_expected() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        // For period=3 and values [1,2,3,4,5], expected EMA outputs are [2.0, 3.0, 4.0]
        assert_eq!(ema(&values, 3), Ok(vec![2.0, 3.0, 4.0]));
    }
}
