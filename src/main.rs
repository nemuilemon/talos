use futures_util::StreamExt;
use serde::Deserialize;
use std::error::Error;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Talos 24h Ticker struct for json resp
#[allow(dead_code)]
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

#[derive(Debug, Deserialize)]
struct WsTicker {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    last_price: String,
    #[serde(rename = "b")]
    bid_price: String,
    #[serde(rename = "a")]
    ask_price: String,
    #[serde(rename = "P")]
    price_change_percent: String,
    #[serde(rename = "h")]
    high_price: String,
    #[serde(rename = "l")]
    low_price: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "wss://stream.binance.com:9443/ws/ethusdt@ticker";

    loop {
        let (mut ws, _response) = match connect_async(url).await {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("connect failed: {e}");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue; // back to the top of the outer loop
            }
        };

        let mut prev: Option<f64> = None;
        while let Some(msg) = ws.next().await {
            let msg = match msg {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("recv failed: {e}");
                    break;
                }
            };

            match msg {
                Message::Text(text) => {
                    let t: WsTicker = serde_json::from_str(&text)?;
                    let last: f64 = t.last_price.parse()?;
                    let bid: f64 = t.bid_price.parse()?;
                    let ask: f64 = t.ask_price.parse()?;
                    let pct: f64 = t.price_change_percent.parse()?;
                    let high: f64 = t.high_price.parse()?;
                    let low: f64 = t.low_price.parse()?;

                    let arrow = match prev {
                        None => " ",
                        Some(p) if last > p => "↑",
                        Some(p) if last < p => "↓",
                        Some(_) => "→",
                    };
                    prev = Some(last);

                    println!(
                        "{}: ${:.2} {} (bid=${:.2} / ask=${:.2}, 24h={:+.2}%, H=${:.2} / L=${:.2})",
                        t.symbol, last, arrow, bid, ask, pct, high, low
                    );
                }
                Message::Close(frame) => {
                    println!("closed: {frame:?}");
                    break;
                }
                _ => {}
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
