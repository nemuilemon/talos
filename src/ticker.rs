use std::error::Error;

use serde::Deserialize;

// Raw message from the Binance @ticker stream (prices come as strings)
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

// WsTicker converted to f64 so we can calculate with it
#[derive(Debug, Clone)]
pub struct Tick {
    pub symbol: String,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub pct: f64,
    pub high: f64,
    pub low: f64,
}

pub fn parse_tick(text: &str) -> Result<Tick, Box<dyn Error>> {
    let t: WsTicker = serde_json::from_str(text)?;
    Ok(Tick {
        symbol: t.symbol,
        last: t.last_price.parse()?,
        bid: t.bid_price.parse()?,
        ask: t.ask_price.parse()?,
        pct: t.price_change_percent.parse()?,
        high: t.high_price.parse()?,
        low: t.low_price.parse()?,
    })
}
