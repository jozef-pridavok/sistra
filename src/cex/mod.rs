use anyhow::Result;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};

use crate::{
    cex::{kucoin::KucoinClient, okx::OkxClient, simulate::SimulateClient},
    coin::Coin,
    config::Config,
    order::{OrderResponse, Side},
};

pub mod kucoin;
pub mod okx;
pub mod simulate;

/// Trait that governs all clients for centralized exchanges
#[async_trait]
pub trait CexClient: Send + Sync {
    /// Gets the current price of the symbol in USDT
    async fn get_price(&self, coin: &Coin) -> Result<f64>;

    /// Gets historical prices of the symbol for the last `days` days, oldest first
    async fn get_historical(&self, coin: &Coin, days: u32) -> Result<Vec<f64>>;

    // Places a market order on the exchange
    async fn put_order(&self, coin: &Coin, side: Side, amount: f64, price: Option<f64>) -> Result<OrderResponse>;
}

pub fn create_cex_client_from_config(config: &Config) -> Result<Box<dyn CexClient>> {
    match config.cex.to_lowercase().as_str() {
        "kucoin" => Ok(Box::new(KucoinClient::new(
            config.cex_api_key.clone(),
            config.cex_api_secret.clone(),
            config.cex_api_passphrase.clone(),
        ))),
        "okx" => Ok(Box::new(OkxClient::new(
            config.cex_api_key.clone(),
            config.cex_api_secret.clone(),
            config.cex_api_passphrase.clone(),
            true,
        ))),
        "simulate" => {
            let simulate_int: u32 = config.simulate_day; // 20250722
            let year = (simulate_int / 10_000) as i32; // 2025
            let month = (simulate_int / 100) % 100; // 07
            let day = simulate_int % 100; // 22

            let start_date = NaiveDate::from_ymd_opt(year, month, day).expect("invalid simulate_day");

            //let today = Utc::today().naive_utc();
            let today = Utc::now().date_naive();

            let days_back = (today - start_date).num_days();
            if days_back < 0 {
                anyhow::bail!("simulate_day is in the future!");
            }

            if let Ok(simulate) = SimulateClient::new(config.simulate_file.clone(), days_back as u32) {
                return Ok(Box::new(simulate));
            }
            Err(anyhow::anyhow!("Invalid simulate client"))
        }
        other => Err(anyhow::format_err!("Unsupported exchange: {}", other)),
    }
}

// eof
