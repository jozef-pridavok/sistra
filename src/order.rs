#[derive(Debug)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct OrderResponse {
    //pub order_id: String,
    pub executed_price: f64,
    pub executed_amount: f64,
    //pub amount: f64,
    pub btc_fee: f64,
    pub usd_fee: f64,
}

/*
pub async fn put_order(
    _side: Side,
    amount: f64,
    price: f64,
) -> Result<OrderResponse> {
    // hard-coded fee 0,2%
    Ok(OrderResponse {
        //order_id: "order_123".into(),
        executed_price: price,
        //amount,
        btc_fee: 0.0, //amount * 0.002, // 0.2% fee
        usd_fee: amount * price * 0.002, // 0.2% fee
    })
}
*/

impl std::fmt::Display for OrderResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:.2} USD. Fees: fee: {:.8}, {:.2} USD",
            self.executed_price, self.btc_fee, self.usd_fee
        )
    }
}

// eof
