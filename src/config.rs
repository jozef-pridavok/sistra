use anyhow::Result;
use clap::{Parser, ValueEnum};
use serde::Deserialize;

use crate::{coin::Coin, logger::LogLevel, strategy::Strategy};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ValueEnum)]
pub enum PerfFeeMode {
    HighWaterMark,
    Cumulative,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    #[arg(long)]
    pub log_level: Option<LogLevel>,

    #[arg(long)]
    pub coin: Option<Coin>,
    #[arg(long)]
    pub period: Option<u32>,
    #[arg(long)]
    pub initial_btc: Option<f64>,
    #[arg(long)]
    pub initial_usd: Option<f64>,
    #[arg(long)]
    pub allocation: Option<f64>,

    #[arg(long)]
    pub take_profit_btc: Option<f64>,
    #[arg(long)]
    pub stop_lose_btc: Option<f64>,

    #[arg(long)]
    pub take_profit_usd: Option<f64>,
    #[arg(long)]
    pub stop_lose_usd: Option<f64>,

    #[arg(long)]
    pub use_fear_index: Option<bool>,

    #[arg(long)]
    pub strategy: Option<Strategy>,

    #[arg(long)]
    pub ema_short: Option<usize>,
    #[arg(long)]
    pub ema_long: Option<usize>,

    #[arg(long)]
    pub rsi_period: Option<usize>,
    #[arg(long)]
    pub rsi_oversold: Option<f64>,
    #[arg(long)]
    pub rsi_overbought: Option<f64>,

    #[arg(long)]
    pub grid_levels: Option<usize>,
    #[arg(long)]
    pub grid_range: Option<f64>,

    #[arg(long)]
    pub dip_pct: Option<f64>,

    #[arg(long)]
    pub tp_pct: Option<f64>,
    #[arg(long)]
    pub buyback_pct: Option<f64>,

    #[arg(long)]
    pub weight_ema: Option<f64>,
    #[arg(long)]
    pub weight_rsi: Option<f64>,
    #[arg(long)]
    pub weight_grid: Option<f64>,
    #[arg(long)]
    pub weight_buy_dip: Option<f64>,
    #[arg(long)]
    pub weight_tp_o_bb: Option<f64>,

    #[arg(long)]
    pub signal_threshold: Option<f64>,

    #[arg(long)]
    pub perf_fee_cycles: Option<u32>,
    #[arg(long)]
    pub perf_fee_rate: Option<f64>,
    #[arg(long)]
    pub perf_fee_mode: Option<PerfFeeMode>,
    #[arg(long)]
    pub deduct_fee_from_balance: Option<bool>,

    #[arg(long)]
    pub cex: Option<String>,
    #[arg(long)]
    pub cex_api_passphrase: Option<String>,
    #[arg(long)]
    pub cex_api_secret: Option<String>,
    #[arg(long)]
    pub cex_api_key: Option<String>,

    #[arg(long)]
    pub simulate_file: Option<String>,
    #[arg(long)]
    pub simulate_day: Option<u32>,
    #[arg(long)]
    pub simulate_cycles: Option<u32>,

    #[arg(long)]
    pub telegram_channel_id: Option<i64>,
    //     #[arg(long, short = 'v', long, default_value_t = false, help = "Print version")]
    //     pub version: bool,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub log_level: LogLevel,
    pub coin: Coin,
    pub period: u32,
    pub initial_btc: f64,
    pub initial_usd: f64,
    pub allocation: f64,

    pub take_profit_btc: f64,
    pub stop_lose_btc: f64,

    pub take_profit_usd: f64,
    pub stop_lose_usd: f64,

    pub strategy: Strategy,
    pub use_fear_index: bool,

    pub ema_short: usize,
    pub ema_long: usize,

    pub rsi_period: usize,
    pub rsi_oversold: f64,
    pub rsi_overbought: f64,

    pub grid_levels: usize,
    pub grid_range: f64,

    pub dip_pct: f64,

    pub tp_pct: f64,
    pub buyback_pct: f64,

    pub weight_ema: f64,
    pub weight_rsi: f64,
    pub weight_grid: f64,
    pub weight_buy_dip: f64,
    pub weight_tp_o_bb: f64,

    pub signal_threshold: f64,

    pub perf_fee_cycles: u32,
    pub perf_fee_rate: f64,
    pub perf_fee_mode: PerfFeeMode,
    pub deduct_fee_from_balance: bool,

    pub cex: String,
    pub cex_api_passphrase: String,
    pub cex_api_secret: String,
    pub cex_api_key: String,

    pub simulate_file: String,
    pub simulate_day: u32,
    pub simulate_cycles: u32,

    pub telegram_channel_id: i64,

    #[serde(skip)]
    pub is_simulation: bool,
}

impl Config {
    pub fn load_from_args() -> Result<Self> {
        let cli = Cli::parse();

        // if cli.version {
        //     cli.print_version();
        //     return Err(anyhow::anyhow!("Done"));
        // }

        let content = match std::fs::read_to_string(&cli.config) {
            Ok(content) => content,
            Err(e) => panic!("Nepodarilo sa načítať konfiguračný súbor. {e}"),
        };
        let mut cfg: Config = toml::from_str(&content)?;

        macro_rules! override_opt {
            ($field:ident, $opt:expr) => {
                if let Some(val) = $opt {
                    cfg.$field = val;
                }
            };
        }

        override_opt!(log_level, cli.log_level);

        override_opt!(coin, cli.coin);
        override_opt!(period, cli.period);
        override_opt!(initial_btc, cli.initial_btc);
        override_opt!(initial_usd, cli.initial_usd);
        override_opt!(allocation, cli.allocation);

        override_opt!(take_profit_btc, cli.take_profit_btc);
        override_opt!(stop_lose_btc, cli.stop_lose_btc);

        override_opt!(take_profit_usd, cli.take_profit_usd);
        override_opt!(stop_lose_usd, cli.stop_lose_usd);

        override_opt!(strategy, cli.strategy);
        override_opt!(use_fear_index, cli.use_fear_index);

        override_opt!(ema_short, cli.ema_short);
        override_opt!(ema_long, cli.ema_long);

        override_opt!(rsi_period, cli.rsi_period);
        override_opt!(rsi_oversold, cli.rsi_oversold);
        override_opt!(rsi_overbought, cli.rsi_overbought);

        override_opt!(grid_levels, cli.grid_levels);
        override_opt!(grid_range, cli.grid_range);

        override_opt!(dip_pct, cli.dip_pct);

        override_opt!(tp_pct, cli.tp_pct);
        override_opt!(buyback_pct, cli.buyback_pct);

        override_opt!(weight_ema, cli.weight_ema);
        override_opt!(weight_rsi, cli.weight_rsi);
        override_opt!(weight_grid, cli.weight_grid);
        override_opt!(weight_buy_dip, cli.weight_buy_dip);
        override_opt!(weight_tp_o_bb, cli.weight_tp_o_bb);

        override_opt!(signal_threshold, cli.signal_threshold);

        override_opt!(perf_fee_cycles, cli.perf_fee_cycles);
        override_opt!(perf_fee_rate, cli.perf_fee_rate);
        override_opt!(perf_fee_mode, cli.perf_fee_mode);
        override_opt!(deduct_fee_from_balance, cli.deduct_fee_from_balance);

        override_opt!(cex, cli.cex);
        override_opt!(cex_api_passphrase, cli.cex_api_passphrase);
        override_opt!(cex_api_secret, cli.cex_api_secret);
        override_opt!(cex_api_key, cli.cex_api_key);

        override_opt!(simulate_file, cli.simulate_file);
        override_opt!(simulate_day, cli.simulate_day);
        override_opt!(simulate_cycles, cli.simulate_cycles);

        override_opt!(telegram_channel_id, cli.telegram_channel_id);

        cfg.is_simulation = cfg.cex.eq_ignore_ascii_case("simulate"); // && cfg.simulate_cycles > 30;
        // if cfg.is_simulation {
        //     if cfg.simulate_cycles > 30 {
        //         cfg.log_level = LogLevel::Error;
        //     }
        // }

        Ok(cfg)
    }
}

impl Config {
    pub fn is_simulation(&self) -> bool {
        self.is_simulation
    }
}

// eof
