use anyhow::Result;
use clap::ValueEnum;
use serde::Deserialize;

use crate::{
    config::Config,
    signal::{Signal, Signals},
};

#[derive(Debug, Clone, Copy, ValueEnum, Deserialize, PartialEq)]
pub enum Strategy {
    Ema,
    Rsi,
    Grid,
    BuyDip,
    TpOBb,
    Combined,
}

#[derive(Debug, Clone, Copy)]
pub struct StrategyWeights {
    pub ema: f64,
    pub rsi: f64,
    pub grid: f64,
    pub buy_dip: f64,
    pub tp_or_bb: f64,
}

/// -- GRID STRATEGY --
/// Splits the interval into grids and generates BUY/SELL signals based on the threshold
pub fn _grid_strategy(prices: &[f64], grid_size: usize) -> Signal {
    if prices.is_empty() || grid_size == 0 {
        return Signal::Hold;
    }
    let min = *prices.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max = *prices.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let step = (max - min) / grid_size as f64;
    let mut last_grid = ((prices[0] - min) / step).floor() as i64;
    let mut signals = vec![];

    for &p in prices.iter().skip(1) {
        let grid = ((p - min) / step).floor() as i64;
        if grid < last_grid {
            signals.push(Signal::Buy);
        } else if grid > last_grid {
            signals.push(Signal::Sell);
        }
        last_grid = grid;
    }
    signals.last().cloned().unwrap_or(Signal::Hold)
}

pub fn grid_strategy(prices: &[f64], grid_size: usize, range: f64) -> Signal {
    if prices.is_empty() || grid_size == 0 || range <= 0.0 {
        return Signal::Hold;
    }
    let latest = *prices.last().unwrap();
    let min = latest * (1.0 - range);
    let max = latest * (1.0 + range);
    let step = (max - min) / grid_size as f64;
    if step <= 0.0 {
        return Signal::Hold;
    }
    let clamp = |i: i64| -> i64 { i.max(0).min(grid_size as i64 - 1) };

    //let mut last_grid = ((prices[0] - min) / step).floor() as i64;
    let mut last_grid = clamp(((prices[0].clamp(min, max) - min) / step).floor() as i64);
    let mut signals = vec![];

    for &p in prices.iter().skip(1) {
        //let grid = ((p - min) / step).floor() as i64;
        let grid = clamp(((p.clamp(min, max) - min) / step).floor() as i64);
        if grid < last_grid {
            signals.push(Signal::Buy);
        } else if grid > last_grid {
            signals.push(Signal::Sell);
        }
        last_grid = grid;
    }
    signals.last().cloned().unwrap_or(Signal::Hold)
}

/// -- EMA Crossover STRATEGY --
pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() || period == 0 {
        return Vec::new();
    }
    let mut result = vec![];
    let k = 2.0 / (period as f64 + 1.0);
    //let mut ema_prev = data[0];
    let mut ema_prev = if data.len() >= period {
        data[..period].iter().sum::<f64>() / period as f64
    } else {
        data[0]
    };
    result.push(ema_prev);
    for &p in data.iter().skip(1) {
        ema_prev = (p - ema_prev) * k + ema_prev;
        result.push(ema_prev);
    }
    result
}

/// Returns signals according to EMA crossover (short/long)
pub fn ema_crossover_strategy(prices: &[f64], short: usize, long: usize) -> Signal {
    if prices.len() < long || short == 0 || long == 0 || short >= long {
        return Signal::Hold;
    }
    let ema_short = ema(prices, short);
    let ema_long = ema(prices, long);
    let mut signals = vec![Signal::Hold];
    for i in 1..prices.len() {
        if ema_short[i - 1] <= ema_long[i - 1] && ema_short[i] > ema_long[i] {
            signals.push(Signal::Buy);
        } else if ema_short[i - 1] >= ema_long[i - 1] && ema_short[i] < ema_long[i] {
            signals.push(Signal::Sell);
        } else {
            signals.push(Signal::Hold);
        }
    }
    signals.last().cloned().unwrap_or(Signal::Hold)
}

/// -- RSI STRATEGY --
pub fn rsi(prices: &[f64], period: usize) -> Vec<f64> {
    let mut rsis = vec![50.0; prices.len()];
    if prices.len() <= period {
        return rsis;
    }
    let mut gains = 0.0;
    let mut losses = 0.0;
    for i in 1..=period {
        let diff = prices[i] - prices[i - 1];
        if diff > 0.0 {
            gains += diff;
        } else {
            losses -= diff;
        }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;
    rsis[period] = if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
    };
    for i in period + 1..prices.len() {
        let diff = prices[i] - prices[i - 1];
        if diff > 0.0 {
            avg_gain = (avg_gain * (period as f64 - 1.0) + diff) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0)) / period as f64;
        } else {
            avg_gain = (avg_gain * (period as f64 - 1.0)) / period as f64;
            avg_loss = (avg_loss * (period as f64 - 1.0) - diff) / period as f64;
        }
        rsis[i] = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
        };
    }
    rsis
}

pub fn rsi_strategy(prices: &[f64], period: usize, oversold: f64, overbought: f64) -> Signal {
    let rsi_vals = rsi(prices, period);
    prices
        .iter()
        .enumerate()
        .map(|(i, _)| {
            if rsi_vals[i] < oversold {
                Signal::Buy
            } else if rsi_vals[i] > overbought {
                Signal::Sell
            } else {
                Signal::Hold
            }
        })
        .collect::<Vec<_>>()
        .last()
        .cloned()
        .unwrap_or(Signal::Hold)
}

/// -- BUY THE DIP STRATEGY --
pub fn buy_the_dip_strategy(prices: &[f64], dip_pct: f64) -> Signal {
    if prices.len() < 2 || dip_pct <= 0.0 {
        return Signal::Hold;
    }
    let mut signal = Signal::Hold;
    for i in 1..prices.len() {
        if prices[i - 1] > 0.0 {
            let pct = (prices[i] - prices[i - 1]) / prices[i - 1] * 100.0;
            if pct <= -dip_pct {
                signal = Signal::Buy;
            }
        }
    }
    signal
}

/// -- PARTIAL TAKE-PROFIT/BUYBACK STRATEGY --
pub fn partial_take_profit_strategy(prices: &[f64], tp_pct: f64, buyback_pct: f64) -> Signal {
    let mut entry = prices[0];
    let mut signals = vec![Signal::Hold];
    for &price in prices.iter().skip(1) {
        if price >= entry * (1.0 + tp_pct / 100.0) {
            signals.push(Signal::Sell);
            entry = price;
        } else if price <= entry * (1.0 - buyback_pct / 100.0) {
            signals.push(Signal::Buy);
            entry = price;
        } else {
            signals.push(Signal::Hold);
        }
    }
    signals.last().cloned().unwrap_or(Signal::Hold)
}

pub fn generate_signals(cfg: &Config, historical: &[f64], weights: StrategyWeights) -> Result<Signals> {
    if historical.is_empty() {
        anyhow::bail!("No historical data provided");
    }

    let ema_signal: Option<Signal> = if cfg.strategy == Strategy::Ema || weights.ema > 0.0 {
        Some(ema_crossover_strategy(historical, cfg.ema_short, cfg.ema_long))
    } else {
        None
    };
    let rsi_signal: Option<Signal> = if cfg.strategy == Strategy::Rsi || weights.rsi > 0.0 {
        Some(rsi_strategy(
            historical,
            cfg.rsi_period,
            cfg.rsi_oversold,
            cfg.rsi_overbought,
        ))
    } else {
        None
    };
    let grid_signal: Option<Signal> = if cfg.strategy == Strategy::Grid || weights.grid > 0.0 {
        Some(grid_strategy(historical, cfg.grid_levels, cfg.grid_range))
    } else {
        None
    };
    let buy_dip_signal: Option<Signal> = if cfg.strategy == Strategy::BuyDip || weights.buy_dip > 0.0 {
        Some(buy_the_dip_strategy(historical, cfg.dip_pct)) // 5.0 -5% za deň
    } else {
        None
    };
    let partial_take_profit_signal: Option<Signal> = if cfg.strategy == Strategy::TpOBb || weights.tp_or_bb > 0.0 {
        Some(partial_take_profit_strategy(historical, cfg.tp_pct, cfg.buyback_pct)) // 10.0, 10.0 +10% predaj, -10% nákup (10% TP, 10% buyback)
    } else {
        None
    };

    Ok(Signals {
        ema: ema_signal,
        rsi: rsi_signal,
        grid: grid_signal,
        buy_dip: buy_dip_signal,
        tp_o_bb: partial_take_profit_signal,
    })
}

// eof
