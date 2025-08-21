use log::{error, warn};
use teloxide::prelude::*;

pub struct Telegram {
    pub bot: Option<Bot>,
}

impl Telegram {
    pub fn new() -> Self {
        let bot = match std::env::var("TELOXIDE_TOKEN") {
            Ok(token) => Some(Bot::new(token)),
            Err(_) => {
                warn!("Telegram is not configured. Missing TELOXIDE_TOKEN");
                None
            }
        };
        Telegram { bot }
    }

    pub async fn send_message(&self, channel_id: i64, message: &str) {
        if channel_id == 0 {
            return;
        }
        if let Some(bot) = &self.bot {
            let channel = ChatId(channel_id);
            match bot.send_message(channel, message).send().await {
                Ok(_) => {}
                Err(e) => error!("Failed to send message! {e}"),
            }
        }
    }
}

impl Default for Telegram {
    fn default() -> Self {
        Self::new()
    }
}
