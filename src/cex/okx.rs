use anyhow::Result;
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use chrono::{Duration, SecondsFormat, Utc};
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

const OKX_LIVE: &str = "https://www.okx.com";

type HmacSha256 = Hmac<Sha256>;

pub struct OkxClient {
    api_key: String,
    secret: String,
    passphrase: String,
    base: String,
    client: Client,
    is_demo: bool,
}

impl OkxClient {
    pub fn new(api_key: String, secret: String, passphrase: String, is_demo: bool) -> Self {
        OkxClient {
            api_key,
            secret,
            passphrase,
            client: Client::builder().user_agent("okx-rust-client/0.1").build().unwrap(),
            base: OKX_LIVE.to_string(),
            is_demo,
            //base: if demo { OKX_TEST.to_string() } else { OKX_LIVE.to_string() },
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

    async fn get_order_details(&self, inst: &str, ord_id: &str) -> Result<OkxOrderDetailsData> {
        let endpoint = format!("/api/v5/trade/order?instId={inst}&ordId={ord_id}");

        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let sign = self.sign("GET", &endpoint, "", &ts);

        let url = format!("{}{}", self.base, endpoint);
        let mut req = self
            .client
            .get(&url)
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", sign)
            .header("OK-ACCESS-TIMESTAMP", ts.clone())
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase);

        if self.is_demo {
            req = req.header("X-SIMULATED-TRADING", "1");
        }

        let res = req.send().await?;
        let json = res.json::<OkxOrderDetailsResponse>().await?;
        if let Some(data) = json.data {
            let detail = data
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::format_err!("Empty OKX order details response. Error code {}", json.code))?;
            Ok(detail)
        } else {
            Err(anyhow::format_err!("Error code {} Endpoint {}", json.code, endpoint))
        }
    }
}

#[async_trait]
impl CexClient for OkxClient {
    async fn get_price(&self, coin: &Coin) -> Result<f64> {
        let inst = format!("{}-USDT", coin.symbol());
        let url = format!("{}/api/v5/market/ticker?instId={}", self.base, inst);
        debug!("GET {}", url);
        let resp: TickerResp = self.client.get(&url).send().await?.json().await?;
        let t = resp.data.first().ok_or_else(|| anyhow::anyhow!("Empty OKX ticker"))?;
        Ok(t.last.parse()?)
    }

    async fn get_historical(&self, coin: &Coin, days: u32) -> Result<Vec<f64>> {
        // End = today's midnight UTC minus 1 second (i.e., end of yesterday)
        let today_midnight = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .expect("valid time")
            .and_local_timezone(Utc)
            .unwrap();
        let end_ts = (today_midnight - Duration::seconds(1)).timestamp_millis();

        // Start = end_ts minus number of days
        let start_ts = (today_midnight - Duration::days(days as i64) - Duration::seconds(1)).timestamp_millis();

        let inst = format!("{}-USDT", coin.symbol());
        let url = format!(
            "{}/api/v5/market/candles?instId={}&bar=1D&after={}&before={}&limit=300",
            self.base, inst, end_ts, start_ts,
        );
        debug!("GET {}", url);
        let resp: CandleResp = self.client.get(&url).send().await?.json().await?;

        let mut data = resp.data;
        // Sort by timestamp (c[0]), oldest first
        data.sort_by(|a, b| a[0].parse::<i64>().unwrap().cmp(&b[0].parse::<i64>().unwrap()));

        // let closes = resp
        //     .data
        //     .iter()
        //     .map(|c| c[4].parse().unwrap())
        //     .collect();

        let prices = data
            .into_iter()
            .map(
                |c| {
                    (c[2].parse::<f64>().expect("Failed to parse highest price")
                        + c[3].parse::<f64>().expect("Failed to parse lower price"))
                        / 2.0
                }, // Average of high and low price
            )
            .collect();

        Ok(prices)
    }

    async fn put_order(&self, coin: &Coin, side: Side, amount: f64, _price: Option<f64>) -> Result<OrderResponse> {
        let inst_id = format!("{}-USDT", coin.symbol());
        let endpoint = "/api/v5/trade/order";
        //let ts = Utc::now().to_rfc3339();
        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);

        let req = OkxPutOrderRequest {
            inst_id: &inst_id,
            td_mode: "cash",
            side: match side {
                Side::Buy => "buy",
                Side::Sell => "sell",
            },
            ord_type: "market",
            tgt_ccy: "base_ccy",
            sz: amount.to_string(),
        };

        let body = serde_json::to_string(&req)?;
        let sign = self.sign("POST", endpoint, &body, &ts);

        let url = format!("{}{}", self.base, endpoint);
        let mut req = self
            .client
            .post(&url)
            .header("OK-ACCESS-KEY", &self.api_key)
            .header("OK-ACCESS-SIGN", sign)
            .header("OK-ACCESS-TIMESTAMP", ts)
            .header("OK-ACCESS-PASSPHRASE", &self.passphrase)
            .header("Content-Type", "application/json")
            .header("X-SIMULATED-TRADING", "1")
            .body(body);

        if self.is_demo {
            req = req.header("X-SIMULATED-TRADING", "1");
        }

        let res = req.send().await?;

        let json = res.json::<OkxPutOrderResponse>().await?;
        let data = json
            .data
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Empty OKX order response"))?;
        if json.code != "0" {
            return Err(anyhow::format_err!(
                "Príkaz sa nepodarilo zadať. OKX error: {}",
                json.code
            ));
        }

        // Fetch order details to get fees
        let details: OkxOrderDetailsData = self.get_order_details(&inst_id, &data.ord_id).await?;
        let executed_price: f64 = details.fill_px.parse()?;
        let executed_amount: f64 = details.fill_sz.parse()?;
        let btc_fee = if details.fee_ccy == "BTC" {
            details.fee.parse()?
        } else {
            0.0
        };
        let usd_fee = if details.fee_ccy == "USDT" {
            details.fee.parse()?
        } else {
            0.0
        };

        Ok(OrderResponse {
            executed_price,
            executed_amount,
            btc_fee,
            usd_fee,
        })
    }
}

#[derive(Deserialize)]
struct TickerResp {
    data: Vec<TickerData>,
}

#[derive(Deserialize)]
struct TickerData {
    //instId: String,
    last: String,
}

#[derive(Deserialize)]
struct CandleResp {
    data: Vec<Vec<String>>,
}

#[derive(Serialize)]
struct OkxPutOrderRequest<'a> {
    #[serde(rename = "instId")]
    inst_id: &'a str,

    #[serde(rename = "tdMode")]
    td_mode: &'a str,

    side: &'a str,

    #[serde(rename = "ordType")]
    ord_type: &'a str,

    #[serde(rename = "tgtCcy")]
    tgt_ccy: &'a str,

    sz: String,
}

#[derive(Deserialize)]
struct OkxPutOrderResponse {
    code: String,
    data: Vec<OkxPutOrderData>,
}

#[derive(Deserialize)]
struct OkxPutOrderData {
    //fillPx: String,
    //fee: String,
    #[serde(rename = "ordId")]
    ord_id: String,
}

#[derive(Deserialize)]
struct OkxOrderDetailsData {
    #[serde(rename = "fillPx")]
    fill_px: String,

    #[serde(rename = "fillSz")]
    fill_sz: String,

    fee: String,

    #[serde(rename = "feeCcy")]
    fee_ccy: String,
}

#[derive(Deserialize)]
struct OkxOrderDetailsResponse {
    code: String,
    data: Option<Vec<OkxOrderDetailsData>>,
}

// eof
