use sistra::{cex::{kucoin::KucoinClient, CexClient}, config::Config, logger::setup_logger};

//  cargo r --example kucoin -- --config ./examples/config.toml

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::load_from_args()?;
    setup_logger(cfg.log_level);

    let kucoin = KucoinClient::new("api".to_string(), "sec".to_string(), "pass".to_string());

    let price = kucoin.get_price(&cfg.coin).await?;
    println!("OKX actual price: {:.8} {}", price, cfg.coin.symbol());

    let historical = kucoin.get_historical(&cfg.coin, 365).await?;
    print_historical(&historical);

    println!("Done");    
    Ok(())
}

fn print_historical(historical: &[f64]) {
    let today = chrono::Utc::now().date_naive();
    for (i, price) in historical.iter().enumerate() {
        let date = today - chrono::Duration::days((historical.len() - i) as i64);
        println!("  Dátum: {date}, Cena: {price:.2}");
    }
}
