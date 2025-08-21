use log::info;

use crate::balance::Balance;
use crate::config::{Config, PerfFeeMode};
use crate::info_buf;

//const DEFAULT_HWM_WINDOW: usize = 10; // počet cyklov na sledovanie HWM v okne

pub struct PerfFeeTracker {
    // window_btc: Vec<f64>,
    // window_usd: Vec<f64>,
    pub initial_balance: Balance,
    pub deduct_fee_from_balance: bool,
    pub high_water_mark_btc: f64,
    pub high_water_mark_usd: f64,
    pub total_fee_btc: f64,
    pub total_fee_usd: f64,
}

impl PerfFeeTracker {
    pub fn new(initial_balance: &Balance, deduct_from_balance: bool) -> Self {
        Self {
            // window_btc: vec![initial_balance.btc_balance],
            // window_usd: vec![initial_balance.usd_balance],
            initial_balance: initial_balance.clone(),
            high_water_mark_btc: initial_balance.btc_balance,
            high_water_mark_usd: initial_balance.usd_balance,
            total_fee_btc: 0.0,
            total_fee_usd: 0.0,
            deduct_fee_from_balance: deduct_from_balance,
        }
    }

    pub fn maybe_deduct_fee(
        &mut self,
        cfg: &Config,
        balance: &mut Balance,
        //initial_balance: &Balance,
        msgs: &mut Vec<String>,
    ) {
        if cfg.perf_fee_rate == 0.0 || cfg.perf_fee_cycles == 0 {
            return;
        }

        let perf_fee = cfg.perf_fee_rate * 100.0;
        let symbol = cfg.coin.symbol();

        info_buf!(msgs, "Výplata odmeny:");

        match cfg.perf_fee_mode {
            PerfFeeMode::HighWaterMark => {
                // // Uložiť aktuálny stav do okna
                // self.window_btc.push(balance.btc_balance);
                // self.window_usd.push(balance.usd_balance);

                // if self.window_btc.len() > DEFAULT_HWM_WINDOW {
                //     self.window_btc.remove(0);
                // }
                // if self.window_usd.len() > DEFAULT_HWM_WINDOW {
                //     self.window_usd.remove(0);
                // }

                // // nové HWM je max z posledných X cyklov
                // self.high_water_mark_btc = self.window_btc.iter().copied().fold(f64::MIN, f64::max);
                // self.high_water_mark_usd = self.window_usd.iter().copied().fold(f64::MIN, f64::max);

                let profit_btc = (balance.btc_balance - self.high_water_mark_btc).max(0.0);
                if profit_btc > 0.0 {
                    let fee_btc = profit_btc * cfg.perf_fee_rate;
                    if self.deduct_fee_from_balance {
                        balance.btc_balance -= fee_btc;
                    }
                    self.total_fee_btc += fee_btc;
                    info_buf!(msgs, "  - vyplatiť {fee_btc:.8} {symbol} ({perf_fee:.2}%)");
                } else {
                    info_buf!(msgs, "  - žiadna odmena {symbol}");
                }

                let profit_usd = (balance.usd_balance - self.high_water_mark_usd).max(0.0);
                if profit_usd > 0.0 {
                    let fee_usd = profit_usd * cfg.perf_fee_rate;
                    if self.deduct_fee_from_balance {
                        balance.usd_balance -= fee_usd;
                    }
                    self.total_fee_usd += fee_usd;
                    info_buf!(msgs, "  - vyplatiť {fee_usd:.2} USD ({perf_fee:.2}%)");
                } else {
                    info_buf!(msgs, "  - žiadna odmena USD");
                }

                self.high_water_mark_btc = balance.btc_balance;
                self.high_water_mark_usd = balance.usd_balance;
            }

            PerfFeeMode::Cumulative => {
                let total_profit_btc = (balance.btc_balance - self.initial_balance.btc_balance).max(0.0);
                let desired_fee_btc = total_profit_btc * cfg.perf_fee_rate;
                let fee_btc = (desired_fee_btc - self.total_fee_btc).max(0.0);
                if fee_btc > 0.0 {
                    if self.deduct_fee_from_balance {
                        balance.btc_balance -= fee_btc;
                    }
                    self.total_fee_btc += fee_btc;
                    info_buf!(msgs, "  - vyplatiť {fee_btc:.8} {symbol} (kumulatívne, {perf_fee:.2}%)");
                } else {
                    info_buf!(msgs, "  - žiadna odmena {symbol}");
                }

                let total_profit_usd = (balance.usd_balance - self.initial_balance.usd_balance).max(0.0);
                let desired_fee_usd = total_profit_usd * cfg.perf_fee_rate;
                let fee_usd = (desired_fee_usd - self.total_fee_usd).max(0.0);
                if fee_usd > 0.0 {
                    if self.deduct_fee_from_balance {
                        balance.usd_balance -= fee_usd;
                    }
                    self.total_fee_usd += fee_usd;
                    info_buf!(msgs, "  - vyplatiť {fee_usd:.2} USD (kumulatívne, {perf_fee:.2}%)");
                } else {
                    info_buf!(msgs, "  - žiadna odmena USD");
                }
            }
        }

        if cfg.perf_fee_rate > 0.0 && cfg.is_simulation() {
            info!(
                "Celková odmena: {:.8} {symbol}, {:.2} USD",
                self.total_fee_btc, self.total_fee_usd
            );
        }
    }
}
