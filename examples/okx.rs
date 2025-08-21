use sistra::{
    cex::{CexClient, okx::OkxClient},
    config::Config,
    logger::setup_logger,
    order::Side,
};

//  cargo r --example okx -- --config ./examples/config.toml

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::load_from_args()?;
    setup_logger(cfg.log_level);

    let okx = OkxClient::new(cfg.cex_api_key, cfg.cex_api_secret, cfg.cex_api_passphrase, true);

    let price = okx.get_price(&cfg.coin).await?;
    println!("OKX actual price: {:.8} {}", price, cfg.coin.symbol());

    let historical = okx.get_historical(&cfg.coin, 365).await?;
    print_historical(&historical);

    //let res = okx.put_order(&cfg.coin, Side::Buy, 0.01, None).await?;
    let res = okx.put_order(&cfg.coin, Side::Sell, 0.002, None).await?;
    println!("Put order result: {res:?}");

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
