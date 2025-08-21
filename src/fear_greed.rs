use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

use crate::strategy::StrategyWeights;

#[derive(Debug, Deserialize)]
struct FearGreedResponse {
    data: Vec<FearGreedData>,
}

#[derive(Debug, Deserialize)]
struct FearGreedData {
    value: String,
    value_classification: String,
    //timestamp: String,
    //time_until_update: String,
}

pub struct FearGreedIndex {
    pub value: u8,
    pub classification: String,
}

impl FearGreedIndex {
    pub async fn fetch() -> Result<Self> {
        let url = "https://api.alternative.me/fng/";
        let client = Client::new();
        let resp = client.get(url).send().await?.json::<FearGreedResponse>().await?;

        let fg = &resp.data[0];
        Ok(Self {
            value: fg.value.parse::<u8>()?,
            classification: fg.value_classification.clone(),
        })
    }

    pub fn normalize_weight(&self) -> f64 {
        // 0..100 => 0.0..1.0 (extrémny strach -> 0.0, extrémna chamtivosť -> 1.0)
        self.value as f64 / 100.0
    }

    /// Funkcia na úpravu stratégických váh na základe sentimentu trhu
    /*
    Aktuálne správanie:
        - Buy the dip stratégia má najväčšiu váhu pri strachu (1.5×) a naopak nízku pri eufórii (0.5×).
        - TP/BB stratégia má naopak najväčšiu váhu pri chamtivosti.
        - Grid ostáva nemenný.
        - EMA a RSI sa jemne upravujú (viac reagujú na strach/chamtivosť v miernej forme).
    */
    pub fn apply(&self, base: &StrategyWeights) -> StrategyWeights {
        let greed = self.normalize_weight(); // 0.0 (strach) .. 1.0 (chamtivosť)

        StrategyWeights {
            ema: base.ema * linear_scale(0.9, 1.1, greed),
            rsi: base.rsi * linear_scale(1.1, 0.9, greed),
            grid: base.grid * linear_scale(1.0, 1.0, greed), // nezmenené
            buy_dip: base.buy_dip * linear_scale(1.5, 0.5, greed),
            tp_or_bb: base.tp_or_bb * linear_scale(0.5, 1.5, greed),
        }
    }
}

/// Pomocná funkcia na lineárne škálovanie v rozsahu greed
/// low_val = hodnota pri fear=0.0, high_val = hodnota pri greed=1.0
#[inline(always)]
fn linear_scale(low_val: f64, high_val: f64, greed: f64) -> f64 {
    low_val + (high_val - low_val) * greed
}

// eof
