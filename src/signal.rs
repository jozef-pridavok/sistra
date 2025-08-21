#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug)]
pub struct Signals {
    pub ema: Option<Signal>,
    pub rsi: Option<Signal>,
    pub grid: Option<Signal>,
    pub buy_dip: Option<Signal>,
    pub tp_o_bb: Option<Signal>,
}

// eof
