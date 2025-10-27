#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
#![deny(missing_docs, unused_must_use)]

//! Deterministic backtest utilities. Simple example: buy at first close, sell at last close.
//!
//! Contracts: functions return Results for invalid inputs. No panics or unwraps.

/// Price bar for a single timeframe
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PriceBar {
    /// epoch seconds (or arbitrary monotonically increasing index)
    pub ts: u64,
    /// open price for the bar
    pub open: f64,
    /// high price for the bar
    pub high: f64,
    /// low price for the bar
    pub low: f64,
    /// close price for the bar
    pub close: f64,
    /// traded volume for the bar
    pub volume: f64,
}

/// Engine configuration
#[derive(Clone, Copy, Debug)]
pub struct EngineConfig {
    /// per-trade commission as fraction (e.g., 0.001 = 0.1%)
    pub commission_rate: f64,
    /// slippage per side in absolute price units
    pub slippage: f64,
    /// deterministic seed (not used in this simple engine but accepted for interface)
    pub seed: u64,
}

/// Backtest report
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    /// price at which the entry was executed (includes slippage)
    pub entry_price: f64,
    /// price at which the exit was executed (includes slippage)
    pub exit_price: f64,
    /// gross pnl (exit - entry)
    pub gross_pnl: f64,
    /// total commissions applied to both entry and exit
    pub commissions: f64,
    /// total slippage applied (entry+exit)
    pub slippage: f64,
    /// net pnl after commissions and slippage
    pub net_pnl: f64,
}

/// Simulate a buy-hold trade: buy at first close + slippage, sell at last close - slippage.
/// Returns Report or Err if input invalid.
pub fn simulate_buy_hold(bars: &[PriceBar], cfg: EngineConfig) -> Result<Report, &'static str> {
    if bars.len() < 2 {
        return Err("need at least 2 bars");
    }
    let first = bars.first().ok_or("no bars")?;
    let last = bars.last().ok_or("no bars")?;
    let entry_price = first.close + cfg.slippage;
    let exit_price = last.close - cfg.slippage;
    let gross = exit_price - entry_price;
    // commissions on both entry and exit
    let commissions = (entry_price.abs() + exit_price.abs()) * cfg.commission_rate;
    let slippage_total = cfg.slippage * 2.0;
    let net = gross - commissions - slippage_total;
    Ok(Report {
        entry_price,
        exit_price,
        gross_pnl: gross,
        commissions,
        slippage: slippage_total,
        net_pnl: net,
    })
}

/// Safe division returning an error on division by zero.
///
/// Returns `Ok(result)` when `den != 0.0`, otherwise returns `Err("division by zero")`.
pub fn safe_div(num: f64, den: f64) -> Result<f64, &'static str> {
    if den == 0.0 {
        Err("division by zero")
    } else {
        Ok(num / den)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indicators::sma;

    #[test]
    fn deterministic_buy_hold() {
        let bars = [
            PriceBar {
                ts: 1,
                open: 10.0,
                high: 10.2,
                low: 9.9,
                close: 10.0,
                volume: 100.0,
            },
            PriceBar {
                ts: 2,
                open: 11.0,
                high: 11.1,
                low: 10.8,
                close: 11.0,
                volume: 120.0,
            },
        ];
        let cfg = EngineConfig {
            commission_rate: 0.001,
            slippage: 0.01,
            seed: 42,
        };
        // Build expected report
        let entry = 10.0 + 0.01;
        let exit = 11.0 - 0.01;
        let gross = exit - entry;
        let commissions = (entry + exit) * cfg.commission_rate;
        let slippage_total = cfg.slippage * 2.0;
        let net = gross - commissions - slippage_total;
        let expected = Report {
            entry_price: entry,
            exit_price: exit,
            gross_pnl: gross,
            commissions,
            slippage: slippage_total,
            net_pnl: net,
        };
        assert_eq!(simulate_buy_hold(&bars, cfg), Ok(expected));
    }

    #[test]
    fn use_indicator_in_backtest_example() {
        // sanity check: compute sma of close prices and ensure usage possible
        let bars = [
            PriceBar {
                ts: 1,
                open: 1.0,
                high: 1.0,
                low: 1.0,
                close: 1.0,
                volume: 1.0,
            },
            PriceBar {
                ts: 2,
                open: 2.0,
                high: 2.0,
                low: 2.0,
                close: 2.0,
                volume: 1.0,
            },
            PriceBar {
                ts: 3,
                open: 3.0,
                high: 3.0,
                low: 3.0,
                close: 3.0,
                volume: 1.0,
            },
        ];
        let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
        assert_eq!(sma(&closes, 2), Ok(vec![1.5, 2.5]));
    }
}
