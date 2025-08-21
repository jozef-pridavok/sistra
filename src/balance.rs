#[derive(Debug, Clone)]
pub struct Balance {
    pub btc_balance: f64,
    pub usd_balance: f64,

    pub _btc_price: f64,

    // pub price: f64, // Current price of BTC in USD
    // pub btc_value: f64, // Current value of BTC in USD
    pub initial: Option<Box<Balance>>,
}

impl Balance {
    pub fn new(btc_balance: f64, usd_balance: f64, btc_price: f64) -> Self {
        Balance {
            btc_balance,
            usd_balance,
            _btc_price: btc_price,
            initial: None,
        } // , price: 0.0, btc_value: 0.0
    }

    // pub fn to_usd(&self, price: f64) -> f64 {
    //     self.btc_balance * price + self.usd_balance
    // }

    // pub fn to_btc(&self, price: f64) -> f64 {
    //     self.btc_balance + self.usd_balance / price
    // }

    pub fn set_initial(&mut self, initial: Balance) {
        self.initial = Some(Box::new(initial))
    }

    /// Checks if the BTC balance has fallen below the stop-loss threshold.
    /// Return value: `true` if signals should be **stopped** (stop-loss activated).
    pub fn stop_lose_btc(&self, stop_lose_btc: f64) -> bool {
        if stop_lose_btc == 0.0 {
            return false;
        }
        self.initial
            .as_ref()
            .map(|init| self.btc_balance <= init.btc_balance * (1.0 - stop_lose_btc))
            .unwrap_or(false)
    }

    /// Checks if the USD balance has fallen below the stop-loss threshold.
    /// Return value: `true` if signals should be **stopped** (stop-loss activated).
    pub fn stop_lose_usd(&self, stop_lose_usd: f64) -> bool {
        if stop_lose_usd == 0.0 {
            return false;
        }
        self.initial
            .as_ref()
            .map(|init| self.usd_balance <= init.usd_balance * (1.0 - stop_lose_usd))
            .unwrap_or(false)
    }
}

// impl std::fmt::Display for Balance {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "BTC: {:.8}, USD: {:.2}", self.btc_balance, self.usd_balance)
//     }
// }

// eof
