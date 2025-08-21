use anyhow::Result;
use log::{debug, error, trace};

use crate::balance::Balance;
use crate::cex::CexClient;
use crate::config::Config;
use crate::signal::{Signal, Signals};
use crate::strategy::Strategy;
use crate::{info_buf, order};

fn print_signal(strategy: Strategy, signal: Signal, weight: f64) {
    trace!("{strategy:?} ({weight}) {signal:?}");
}

fn combined(cfg: &Config, signals: Signals) -> Signal {
    // Normalize weights and calculate score for each direction (Buy=+1, Sell=-1, Hold=0)

    let mut total_weight = 0.0;
    let mut score = 0.0;

    if let Some(sig) = signals.ema {
        print_signal(Strategy::Ema, sig, cfg.weight_ema);
        total_weight += cfg.weight_ema;
        score += match sig {
            Signal::Buy => cfg.weight_ema,
            Signal::Sell => -cfg.weight_ema,
            Signal::Hold => 0.0,
        };
    }
    if let Some(sig) = signals.rsi {
        print_signal(Strategy::Rsi, sig, cfg.weight_rsi);
        total_weight += cfg.weight_rsi;
        score += match sig {
            Signal::Buy => cfg.weight_rsi,
            Signal::Sell => -cfg.weight_rsi,
            Signal::Hold => 0.0,
        };
    }
    if let Some(sig) = signals.grid {
        print_signal(Strategy::Grid, sig, cfg.weight_grid);
        total_weight += cfg.weight_grid;
        score += match sig {
            Signal::Buy => cfg.weight_grid,
            Signal::Sell => -cfg.weight_grid,
            Signal::Hold => 0.0,
        };
    }
    if let Some(sig) = signals.buy_dip {
        print_signal(Strategy::BuyDip, sig, cfg.weight_buy_dip);
        total_weight += cfg.weight_buy_dip;
        score += match sig {
            Signal::Buy => cfg.weight_buy_dip,
            Signal::Sell => -cfg.weight_buy_dip,
            Signal::Hold => 0.0,
        };
    }
    if let Some(sig) = signals.tp_o_bb {
        print_signal(Strategy::TpOBb, sig, cfg.weight_tp_o_bb);
        total_weight += cfg.weight_tp_o_bb;
        score += match sig {
            Signal::Buy => cfg.weight_tp_o_bb,
            Signal::Sell => -cfg.weight_tp_o_bb,
            Signal::Hold => 0.0,
        };
    }

    if total_weight > 0.0 {
        let normalized = score / total_weight;
        if normalized > cfg.signal_threshold {
            Signal::Buy
        } else if normalized < -cfg.signal_threshold {
            Signal::Sell
        } else {
            Signal::Hold
        }
    } else {
        Signal::Hold
    }
}

pub async fn execute_signals(
    cfg: &Config,
    exch: &dyn CexClient,
    price: f64,
    signals: Signals,
    balance: &mut Balance,
    msgs: &mut Vec<String>,
) -> Result<()> {
    let signal = match cfg.strategy {
        Strategy::Ema => signals.ema.unwrap(),
        Strategy::Rsi => signals.rsi.unwrap(),
        Strategy::Grid => signals.grid.unwrap(),
        Strategy::BuyDip => signals.buy_dip.unwrap(),
        Strategy::TpOBb => signals.tp_o_bb.unwrap(),
        Strategy::Combined => combined(cfg, signals),
    };

    let mut amount = balance.btc_balance * cfg.allocation;
    if amount <= 0.0 {
        amount = (balance.usd_balance / price) * cfg.allocation;
    }
    if let Signal::Buy | Signal::Sell = signal {
        let stop_lose_usd = signal == Signal::Buy && balance.stop_lose_usd(cfg.stop_lose_usd);
        let stop_lose_btc = signal == Signal::Sell && balance.stop_lose_btc(cfg.stop_lose_btc);
        if stop_lose_btc || stop_lose_usd {
            let symbol = cfg.coin.symbol();
            debug!(
                "  {signal:?} {amount:.8} {symbol}: STOP LOSE {}{}{} 🚫",
                if stop_lose_btc { symbol } else { "" },
                if stop_lose_btc && stop_lose_usd { " and " } else { "" },
                if stop_lose_usd { "USD" } else { "" }
            );
        } else {
            execute_signal(cfg, exch, price, signal, amount, balance, msgs).await?;
        }
    }

    Ok(())
}

async fn execute_signal(
    cfg: &Config,
    exch: &dyn CexClient,
    price: f64,
    signal: Signal,
    amount: f64,
    balance: &mut Balance,
    msgs: &mut Vec<String>,
) -> Result<()> {
    let symbol = cfg.coin.symbol();
    info_buf!(msgs, "{signal:?} {amount:.8} {}", cfg.coin.symbol());
    match signal {
        Signal::Buy => {
            match exch.put_order(&cfg.coin, order::Side::Buy, amount, Some(price)).await {
                Ok(res) => {
                    //debug!("  {res}");
                    info_buf!(msgs, "  cena: {:.2} USD", res.executed_price);
                    if res.btc_fee != 0.0 {
                        info_buf!(msgs, "  poplatok: {:.8} {symbol}", res.btc_fee);
                    }
                    if res.usd_fee != 0.0 {
                        info_buf!(msgs, "  poplatok: {:.2} USD", res.usd_fee);
                    }
                    balance.btc_balance += res.executed_amount - res.btc_fee.abs();
                    balance.usd_balance -= res.executed_amount * res.executed_price + res.usd_fee.abs();
                }
                Err(e) => {
                    error!("Failed to place BUY order: {e}");
                }
            }
        }
        Signal::Sell => match exch.put_order(&cfg.coin, order::Side::Sell, amount, Some(price)).await {
            Ok(res) => {
                if res.btc_fee != 0.0 {
                    info_buf!(msgs, "  poplatok: {:.8} {symbol}", res.btc_fee);
                }
                if res.usd_fee != 0.0 {
                    info_buf!(msgs, "  poplatok: {:.2} USD", res.usd_fee);
                }
                balance.btc_balance -= res.executed_amount + res.btc_fee.abs();
                balance.usd_balance += res.executed_amount * res.executed_price - res.usd_fee.abs();
            }
            Err(e) => {
                error!("Failed to place SELL order: {e}");
            }
        },
        Signal::Hold => {}
    }
    Ok(())
}

// eof
