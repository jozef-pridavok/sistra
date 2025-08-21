use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use std::{
    collections::HashMap,
    fs,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    cex::CexClient,
    coin::Coin,
    order::{OrderResponse, Side},
};

pub struct SimulateClient {
    // determines the currently simulated day, at the beginning day == days_back
    day: AtomicU32,
    // determines how many days I am shifted into the past in the data
    // days_back: u32,
    // data contains yyyyMMdd: BTC price
    data: Vec<(String, f64)>,
}

impl SimulateClient {
    pub fn new(file_path: String, days_back: u32) -> Result<Self> {
        let content = fs::read_to_string(&file_path)?;
        let raw: Vec<HashMap<String, f64>> = serde_json::from_str(&content)?;
        let mut data = Vec::with_capacity(raw.len());
        for entry in raw {
            for (date, value) in entry {
                data.push((date, value));
            }
        }
        data.sort_by_key(|(date, _)| date.clone());

        let total = data.len();
        let start_index = total.saturating_sub(days_back as usize);

        Ok(SimulateClient {
            day: AtomicU32::new(start_index as u32 /* AtomicU32::new(days_back) */),
            /*days_back: days_back,*/ data,
        })
    }
}

#[async_trait]
impl CexClient for SimulateClient {
    // the function returns the price from data for the current day (defined in day)... after returning, day is moved forward by one
    async fn get_price(&self, _coin: &Coin) -> Result<f64> {
        let current = self.day.load(Ordering::SeqCst) as usize;
        if current >= self.data.len() {
            anyhow::bail!(
                "Simulovaný index dňa {} je mimo rozsahu dát (max {})",
                current,
                self.data.len() - 1
            );
        }

        let data = self.data[current].clone();
        let day = data.0;
        debug!("Simulate day: {}", day);

        let price = data.1;

        // shift the day by 1
        self.day.fetch_add(1, Ordering::SeqCst);
        Ok(price)
    }

    /// Returns the last `days` days of historical prices (only f64), excluding today.
    async fn get_historical(&self, _coin: &Coin, days: u32) -> Result<Vec<f64>> {
        // current index (today)
        let current = self.day.load(Ordering::SeqCst) as usize;

        let start = current.saturating_sub(days as usize);
        let slice = &self.data[start..current];

        // extract only the price (second item of the tuple)
        let prices: Vec<f64> = slice.iter().map(|(_, price)| *price).collect();
        Ok(prices)
    }

    async fn put_order(&self, _coin: &Coin, _side: Side, amount: f64, price: Option<f64>) -> Result<OrderResponse> {
        Ok(OrderResponse {
            //order_id: "order_123".into(),
            executed_price: price.unwrap(),
            executed_amount: amount,
            btc_fee: 0.0,                             //amount * 0.002, // 0.2% fee
            usd_fee: amount * price.unwrap() * 0.002, // 0.2% fee
        })
    }
}

// eof
