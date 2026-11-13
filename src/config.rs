use std::time::Duration;

// All tunable settings live here.
// Later this can be filled from CLI args or a file instead of `default()`.
#[derive(Debug, Clone)]
pub struct Config {
    pub symbol: String,            // Binance symbol, e.g. "ETHUSDT"
    pub history_len: usize,        // ticks kept for the chart (about one per second)
    pub reconnect_delay: Duration, // wait before reconnecting the WebSocket
    pub channel_capacity: usize,   // buffer between the WebSocket task and the UI
}

impl Default for Config {
    fn default() -> Self {
        Config {
            symbol: String::from("ETHUSDT"),
            history_len: 2000,
            reconnect_delay: Duration::from_secs(5),
            channel_capacity: 64,
        }
    }
}

impl Config {
    // Binance stream names are lowercase: wss://.../ws/ethusdt@ticker
    pub fn ws_url(&self) -> String {
        format!(
            "wss://stream.binance.com:9443/ws/{}@ticker",
            self.symbol.to_lowercase()
        )
    }
}
