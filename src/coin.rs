use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, ValueEnum)]
pub enum Coin {
    Bitcoin,
    Ethereum,
    Solana,
    Usdt,
}

impl Coin {
    pub fn symbol(&self) -> &str {
        match self {
            Coin::Bitcoin => "BTC",
            Coin::Ethereum => "ETH",
            Coin::Solana => "SOL",
            Coin::Usdt => "USDT",
        }
    }

    pub fn _coin_gecko_id(&self) -> &str {
        match self {
            Coin::Bitcoin => "bitcoin",
            Coin::Ethereum => "ethereum",
            Coin::Solana => "solana",
            Coin::Usdt => "tether",
        }
    }

    pub fn _name(&self) -> &str {
        match self {
            Coin::Bitcoin => "Bitcoin",
            Coin::Ethereum => "Ethereum",
            Coin::Solana => "Solana",
            Coin::Usdt => "Tether USD",
        }
    }
}

// eof
