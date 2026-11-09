use serde::Deserialize;
use std::error::Error;
use std::thread;
use std::time::Duration;

// Talos 24h Ticker struct for json resp
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ticker {
    symbol: String,
    bid_price: String,
    ask_price: String,
    last_price: String,
    high_price: String,
    low_price: String,
    volume: String,
    price_change_percent: String,
}
fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://api.binance.com/api/v3/ticker/24hr?symbol=ETHUSDT";
    let client = reqwest::blocking::Client::new();

    loop {
        let body = client.get(url).send()?.text()?;
        let t: Ticker = serde_json::from_str(&body)?;
        
        println!(
            "{}: ${} (bid=${} / ask=${}, 24h={}%)",
            t.symbol, t.last_price, t.bid_price, t.ask_price, t.price_change_percent
        );

        thread::sleep(Duration::from_secs(5));
    }
}

