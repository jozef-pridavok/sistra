use anyhow::Result;
use chrono::Utc;
use config::Config;
use log::{debug, error, info};
use std::time::Duration;
use tokio::time::{self, Instant, sleep_until};

use crate::{
    balance::Balance, cex::create_cex_client_from_config, fear_greed::FearGreedIndex, fee::PerfFeeTracker,
    logger::setup_logger, strategy::StrategyWeights, telegram::Telegram,
};

// extern crate pretty_env_logger;
// #[macro_use]
// extern crate log;

mod balance;
mod cex;
mod coin;
mod config;
mod executor;
mod fear_greed;
mod fee;
mod logger;
mod order;
mod signal;
mod strategy;
mod telegram;

const INTERVAL: u64 = 60 * 60 * 24; // 1 deň v sekundách

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::load_from_args()?;
    setup_logger(cfg.log_level);

    let telegram = Telegram::new();
    let mut msgs: Vec<String> = Vec::new();

    let symbol = cfg.coin.symbol();

    if cfg.is_simulation() {
        println!("Deň štartu simulácie: {}", cfg.simulate_day);
    }

    let exch_client = match create_cex_client_from_config(&cfg) {
        Ok(client) => client,
        Err(e) => {
            return Err(anyhow::format_err!("Error creating client! {}", e));
        }
    };

    let price = match exch_client.get_price(&cfg.coin).await {
        Ok(p) => p,
        Err(e) => {
            error!("Error fetching price: {e}");
            telegram
                .send_message(cfg.telegram_channel_id, &format!("Error fetching price: {e}"))
                .await;
            panic!();
        }
    };

    let initial_usd = if cfg.initial_usd < 0.0 {
        cfg.initial_btc * price
    } else {
        cfg.initial_usd
    };

    let initial_balance = Balance::new(cfg.initial_btc, initial_usd, price);

    info_buf!(msgs, "Current price {:.2} USD", price);
    info_buf!(msgs, "Starting portfolio:");
    info_buf!(
        msgs,
        "  {:.8} {symbol} ({:.2} USD)",
        cfg.initial_btc,
        cfg.initial_btc * price
    );
    info_buf!(msgs, "  {initial_usd:.2} USD ({:.8} {symbol})", cfg.initial_usd / price);

    if cfg.is_simulation() {
        println!("Current price {price:.2} USD");
        println!(
            "Starting portfolio: {:.8} {symbol}, {initial_usd:.2} USD",
            cfg.initial_btc,
        );
    }

    let mut perf_tracker = PerfFeeTracker::new(&initial_balance, cfg.deduct_fee_from_balance);

    let mut total_take_profit_btc = 0_f64;
    let mut total_take_profit_usd = 0_f64;

    let mut prev_balance = initial_balance.clone();
    prev_balance.set_initial(initial_balance.clone());

    let mut cycle_count: u32 = 0;

    debug!("--------------------------------------------------------------------------------");

    if !cfg.is_simulation() {
        let now = Utc::now();
        let next_midnight = now
            .date_naive()
            .succ_opt()
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();
        let dur_until_midnight = (next_midnight - now).to_std().unwrap();
        debug!("Waiting until midnight: {dur_until_midnight:?}");
        sleep_until(Instant::now() + dur_until_midnight).await;
    }

    let mut interval = if cfg.simulate_day != 0 {
        time::interval(Duration::from_millis(5))
    } else {
        time::interval(Duration::from_secs(INTERVAL))
    };
    // Skip the first tick
    interval.tick().await;

    telegram.send_message(cfg.telegram_channel_id, &msgs.join("\n")).await;

    loop {
        cycle_count = cycle_count.wrapping_add(1);
        let is_perf_day = cycle_count % cfg.perf_fee_cycles == 0;

        let mut msgs: Vec<String> = Vec::new();

        //let price = exch_client.get_price(&cfg.coin).await?;
        let price = match exch_client.get_price(&cfg.coin).await {
            Ok(p) => p,
            Err(e) => {
                error!("Error fetching price: {e}");
                telegram
                    .send_message(cfg.telegram_channel_id, &format!("⛔⛔⛔ Error fetching price: {e}"))
                    .await;
                continue;
            }
        };
        info_buf!(
            msgs,
            "Current price {price:.2} USD, cycle {cycle_count}{}",
            if is_perf_day { " 💲" } else { "" }
        );

        let mut balance = prev_balance.clone();
        info_buf!(msgs, "Initial account state:",);
        info_buf!(msgs, "  {:.8} {symbol}", balance.btc_balance,);
        info_buf!(msgs, "  {:.2} USD", balance.usd_balance);

        let historical = match exch_client.get_historical(&cfg.coin, cfg.period).await {
            Ok(data) => data,
            Err(e) => {
                let message = format!("Error fetching historical data: {e:?}");
                error!("{message}");
                telegram.send_message(cfg.telegram_channel_id, "⛔⛔⛔ {message}").await;
                continue;
            }
        };

        let cfg_weights = StrategyWeights {
            ema: cfg.weight_ema,
            rsi: cfg.weight_rsi,
            grid: cfg.weight_grid,
            buy_dip: cfg.weight_buy_dip,
            tp_or_bb: cfg.weight_tp_o_bb,
        };

        let weights = if cfg.use_fear_index {
            match FearGreedIndex::fetch().await {
                Ok(fear_greed) => {
                    info_buf!(
                        msgs,
                        "F&G Index: {}% => {}",
                        fear_greed.value,
                        fear_greed.classification
                    );
                    /*let adjuested_wights =*/
                    fear_greed.apply(&cfg_weights)
                    //; adjuested_wights
                }
                Err(e) => {
                    let message = format!("Error fetching fear&greed index: {e:?}");
                    error!("{message}");
                    telegram.send_message(cfg.telegram_channel_id, "⛔⛔⛔ {message}").await;
                    cfg_weights
                }
            }
        } else {
            cfg_weights
        };

        let signals = strategy::generate_signals(&cfg, &historical, weights)?;

        executor::execute_signals(&cfg, &*exch_client, price, signals, &mut balance, &mut msgs).await?;

        if cfg.perf_fee_rate > 0.0 && is_perf_day {
            perf_tracker.maybe_deduct_fee(&cfg, &mut balance, &mut msgs);
        }
        // if cfg.perf_fee_rate > 0.0 {
        //     info!(
        //         "Celková odmena: {:.8} {symbol}, {:.2} USD",
        //         perf_tracker.total_fee_btc, perf_tracker.total_fee_usd
        //     );
        // }

        // Take profit

        if cfg.take_profit_btc > 0.0 {
            let btc_profit = (balance.btc_balance - initial_balance.btc_balance).max(0.0);
            if btc_profit / initial_balance.btc_balance >= cfg.take_profit_btc {
                balance.btc_balance -= btc_profit;
                total_take_profit_btc += btc_profit;
                let pct = (btc_profit / initial_balance.btc_balance) * 100.0;
                info_buf!(msgs, "Setting aside {:.8} {symbol}", btc_profit);
                info_buf!(msgs, "  - increase {pct:.2}% since start",);
                info_buf!(msgs, "  - total: {:.8} {symbol}", total_take_profit_btc);
                //high_water_mark_btc = balance.btc_balance;
            }
        }

        if cfg.take_profit_usd > 0.0 {
            let usd_profit = (balance.usd_balance - initial_balance.usd_balance).max(0.0);
            if usd_profit / initial_balance.usd_balance >= cfg.take_profit_usd {
                balance.usd_balance -= usd_profit;
                total_take_profit_usd += usd_profit;
                let pct = (usd_profit / initial_balance.usd_balance) * 100.0;
                info_buf!(msgs, "Setting aside {:.2} USD", usd_profit);
                info_buf!(msgs, "  - increase {pct:.2}% since start",);
                info_buf!(msgs, "  - total: {:.2} USD", total_take_profit_usd);
                //high_water_mark_usd = balance.usd_balance;
            }
        }

        prev_balance = balance.clone();

        if (cfg.take_profit_btc > 0.0 || cfg.take_profit_usd > 0.0)
            && (total_take_profit_btc > 0.0 || total_take_profit_usd > 0.0)
        {
            info_buf!(msgs, "Total set aside:");
            if total_take_profit_btc > 0.0 {
                info_buf!(msgs, "  {total_take_profit_btc:.8} {symbol}");
            }
            if total_take_profit_usd > 0.0 {
                info_buf!(msgs, "  {total_take_profit_usd:.2} USD");
            }
        }

        // info_buf!(msgs, "Koncový stav účtu",);
        // info_buf!(msgs, "  {:.8} {symbol}", balance.btc_balance,);
        // info_buf!(msgs, "  {:.2} USD", balance.usd_balance);

        print_overall_evaluation(
            false,
            &cfg,
            price,
            &initial_balance,
            &balance,
            total_take_profit_btc,
            total_take_profit_usd,
            &mut msgs,
        );

        telegram.send_message(cfg.telegram_channel_id, &msgs.join("\n")).await;

        debug!("");
        debug!(
            "Waiting for next interval... {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        debug!("");
        interval.tick().await;

        if cfg.simulate_cycles > 0 && cycle_count > cfg.simulate_cycles {
            perf_tracker.maybe_deduct_fee(&cfg, &mut balance, &mut msgs);

            // take profit

            print_overall_evaluation(
                true,
                &cfg,
                price,
                &initial_balance,
                &balance,
                total_take_profit_btc,
                total_take_profit_usd,
                &mut msgs,
            );
            if cfg.perf_fee_rate > 0.0 {
                println!(
                    "  Total reward {:.8} {symbol}, {:.2} USD",
                    perf_tracker.total_fee_btc, perf_tracker.total_fee_usd
                );
            }
            break;
        }
    }

    info!("Done");
    if cfg.is_simulation() {
        println!();
    }

    return Ok(());
}

fn _print_historical(historical: &[f64]) {
    let today = chrono::Utc::now().date_naive();
    for (i, price) in historical.iter().enumerate() {
        let date = today - chrono::Duration::days((historical.len() - i) as i64);
        debug!("  Dátum: {date}, Cena: {price:.2}");
    }
}

fn _print_cycle_evaluation(price: f64, old_balance: &Balance, new_balance: &Balance) {
    // Total value in BTC
    let old_total_btc = old_balance.btc_balance + old_balance.usd_balance / price;
    let new_total_btc = new_balance.btc_balance + new_balance.usd_balance / price;
    let pct_btc = if old_total_btc > 0.0 {
        (new_total_btc - old_total_btc) / old_total_btc * 100.0
    } else {
        0.0
    };

    // Total value in USD
    let old_total_usd = old_balance.btc_balance * price + old_balance.usd_balance;
    let new_total_usd = new_balance.btc_balance * price + new_balance.usd_balance;
    let pct_usd = if old_total_usd > 0.0 {
        (new_total_usd - old_total_usd) / old_total_usd * 100.0
    } else {
        0.0
    };

    debug!("  {pct_btc:.4}% in BTC, {pct_usd:.4}% in USD");
}

fn pct(initial: f64, current: f64) -> f64 {
    (current - initial) / initial * 100.0
}

#[allow(clippy::too_many_arguments)]
fn print_overall_evaluation(
    print: bool,
    cfg: &Config,
    _price: f64,
    initial_balance: &Balance,
    current_balance: &Balance,
    total_take_profit_btc: f64,
    total_take_profit_usd: f64,
    msgs: &mut Vec<String>,
) {
    let symbol = cfg.coin.symbol();

    let initial_btc = initial_balance.btc_balance;
    let current_btc = current_balance.btc_balance + total_take_profit_btc;
    let pct_current_btc = pct(initial_btc, current_btc);
    //let pct_current_btc = (current_btc - initial_btc) / initial_btc * 100.0;

    let initial_usd = initial_balance.usd_balance;
    let current_usd = current_balance.usd_balance + total_take_profit_usd;
    //let pct_current_usd = (current_usd - initial_usd) / initial_usd * 100.0;
    let pct_current_usd = pct(initial_usd, current_usd);

    // let total_in_usd = current_usd + current_btc * price;
    // let total_in_btc = current_btc + current_usd / price;

    info_buf!(msgs, "Final account state",);
    info_buf!(msgs, "  {:.8} {}, {pct_current_btc:.2}%", current_btc, symbol);
    info_buf!(msgs, "  {:.2} USD, {pct_current_usd:.2}%", current_usd);

    if print {
        println!("  {symbol}: {pct_current_btc:.2}% ({current_btc:.8}), USD: {pct_current_usd:.2}% ({current_usd:.2})");

        let initial_btc_price = initial_balance._btc_price;

        // If I had bought only BTC at the beginning of the year

        let total_btc = initial_balance.btc_balance + initial_balance.usd_balance / initial_btc_price;
        println!("    Buy at start of the year: Total BTC: {total_btc:.8}");

        // let actual_btc_in_usd = total_btc * price;

        // let pct_btc_price = pct(initial_btc_price, price);
        // println!("    BTC price: {initial_btc_price:.8} -> {price:.8} ({pct_btc_price:.2}%)");
        // println!("    End BTC in USD: {actual_btc_in_usd:.2}");

        /*

        TAV

        let initial_tav_in_usd = initial_balance.to_usd(initial_btc_price); // + initial_balance.usd_balance;
        let current_tav_in_usd = current_balance.to_usd(price); // + current_balance.usd_balance;
        let pct_tav_in_usd = pct(initial_tav_in_usd, current_tav_in_usd);

        let initial_tav_in_btc = initial_balance.to_btc(initial_btc_price); // + initial_balance.btc_balance;
        let current_tav_in_btc = current_balance.to_btc(price); // + current_balance.btc_balance;
        let pct_tav_in_btc = pct(initial_tav_in_btc, current_tav_in_btc);


        println!("    TAV in BTC: {initial_tav_in_btc:.2} -> {current_tav_in_btc:.2} ({pct_tav_in_btc:.2}%)");
        */
    }
}

// eof
