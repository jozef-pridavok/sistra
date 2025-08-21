use anyhow::Result;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::{
    cex::CexClient,
    coin::Coin,
    order::{OrderResponse, Side},
};

const BASE_URL: &str = "https://api.kucoin.com";

type HmacSha256 = Hmac<Sha256>;

pub struct KucoinClient {
    api_key: String,
    secret: String,
    passphrase: String,
    client: Client,
}

impl KucoinClient {
    pub fn new(api_key: String, secret: String, passphrase: String) -> Self {
        KucoinClient {
            api_key,
            secret,
            passphrase,
            client: Client::builder().user_agent("kucoin-rust-client/0.1").build().unwrap(),
        }
    }

    fn sign(&self, method: &str, endpoint: &str, body: &str, timestamp: &str) -> String {
        // Prehash string: timestamp + method + requestPath + body
        let prehash = format!("{timestamp}{method}{endpoint}{body}");
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(prehash.as_bytes());
        let result = mac.finalize().into_bytes();
        general_purpose::STANDARD.encode(result)
    }
}

#[async_trait]
impl CexClient for KucoinClient {
    async fn get_price(&self, coin: &Coin) -> Result<f64> {
        let endpoint = format!("/api/v1/market/orderbook/level1?symbol={}-USDT", coin.symbol());
        let url = format!("{BASE_URL}{endpoint}");
        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<ApiResponse<OrderBookLevel1>>()
            .await?;
        let data = resp
            .data
            .ok_or_else(|| anyhow::anyhow!("No data returned from KuCoin API"))?;
        Ok(data.price.parse()?)
    }

    async fn get_historical(&self, coin: &Coin, days: u32) -> Result<Vec<f64>> {
        // End = today's midnight UTC minus 1 second (i.e., end of yesterday)
        let today_midnight = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .expect("valid time")
            .and_local_timezone(Utc)
            .unwrap();
        let end_ts = (today_midnight - Duration::seconds(1)).timestamp();

        // Start = end_ts minus number of days
        let start_ts = (today_midnight - Duration::days(days as i64) - Duration::seconds(1)).timestamp();

        let endpoint = format!(
            "/api/v1/market/candles?symbol={}-USDT&startAt={}&endAt={}&type=1day",
            coin.symbol(),
            start_ts,
            end_ts
        );
        let url = format!("{BASE_URL}{endpoint}");
        debug!("GET: {}", url);

        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<ApiResponse<Vec<Vec<String>>>>()
            .await?;

        let mut data = resp
            .data
            .ok_or_else(|| anyhow::anyhow!("No data returned from KuCoin API"))?;

        // Sort by timestamp (c[0]), oldest first
        data.sort_by(|a, b| a[0].parse::<i64>().unwrap().cmp(&b[0].parse::<i64>().unwrap()));

        // 0 - start time of the candle cycle,
        // 1 - opening price,
        // 2 - closing price,
        // 3 - highest price,
        // 4 - lowest price,
        // 5 - transaction volume (in base asset),
        // 6 - transaction amount (in quote asset)
        // Extrahujeme iba záverečné ceny
        let prices = data
            .into_iter()
            .map(
                |c| {
                    (c[3].parse::<f64>().expect("parsing highest price")
                        + c[4].parse::<f64>().expect("parsing lower price"))
                        / 2.0
                }, // Average of high and low prices
            )
            .collect();

        Ok(prices)
    }

    async fn put_order(&self, coin: &Coin, side: Side, amount: f64, _price: Option<f64>) -> Result<OrderResponse> {
        let inst = format!("{}-USDT", coin.symbol());
        let endpoint = "/api/v1/orders";
        let ts = Utc::now().timestamp().to_string();

        let req = KucoinOrderRequest {
            symbol: &inst,
            side: match side {
                Side::Buy => "buy",
                Side::Sell => "sell",
            },
            type_: "market",
            size: amount.to_string(),
        };
        let body = serde_json::to_string(&req)?;
        let sign = self.sign("POST", endpoint, &body, &ts);

        let url = format!("{BASE_URL}{endpoint}");
        let resp = self
            .client
            .post(&url)
            .header("KC-API-KEY", &self.api_key)
            .header("KC-API-SIGN", sign)
            .header("KC-API-TIMESTAMP", ts)
            .header("KC-API-PASSPHRASE", &self.passphrase)
            .header("KC-API-KEY-VERSION", "2")
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?
            .json::<KucoinOrderResponse>()
            .await?;

        let data = resp.data;
        let executed_price: f64 = data.price.parse()?;
        let executed_amount: f64 = data.size.parse()?;
        let btc_fee: f64 = data.fee.parse()?;
        let usd_fee = btc_fee * executed_price;
        Ok(OrderResponse {
            executed_price,
            executed_amount,
            btc_fee,
            usd_fee,
        })
    }
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    //code: String,
    data: Option<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrderBookLevel1 {
    //best_bid: String,
    //best_ask: String,
    price: String,
    //time: i64,
}

/*
#[derive(Debug)]
struct Candle {
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrderData {
    symbol: String,
    order_id: String,
}

#[derive(Debug)]
pub struct OrderResponse {
    pub symbol: String,
    pub order_id: String,
}
*/

#[derive(Serialize)]
struct KucoinOrderRequest<'a> {
    symbol: &'a str,
    side: &'a str,
    type_: &'a str,
    size: String,
}

#[derive(Deserialize)]
struct KucoinOrderResponseData {
    price: String,
    size: String,
    fee: String,
}

#[derive(Deserialize)]
struct KucoinOrderResponse {
    data: KucoinOrderResponseData,
}

// eof
