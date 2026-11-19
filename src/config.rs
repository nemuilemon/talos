use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use serde::Deserialize;

/// Binance ticker TUI
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Binance symbol, e.g. ETHUSDC or BTCUSDT (overrides the config file)
    pub symbol: Option<String>,

    /// Path to a TOML config file (default: ~/.config/talos/config.toml)
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}

// All tunable settings live here.
// Built in layers: Default -> config file -> CLI args.
#[derive(Debug, Clone)]
pub struct Config {
    pub symbol: String,            // Binance symbol, e.g. "ETHUSDC"
    pub history_len: usize,        // ticks kept for the chart (about one per second)
    pub reconnect_delay: Duration, // wait before reconnecting the WebSocket
    pub channel_capacity: usize,   // buffer between the WebSocket task and the UI
}

// What a config file may contain. Every key is optional.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    symbol: Option<String>,
    history_len: Option<usize>,
    reconnect_delay_secs: Option<u64>,
    channel_capacity: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            symbol: String::from("ETHUSDC"),
            history_len: 2000,
            reconnect_delay: Duration::from_secs(5),
            channel_capacity: 64,
        }
    }
}

impl Config {
    pub fn load(args: Args) -> Result<Config, Box<dyn Error>> {
        let mut config = Config::default();

        // 1. config file: an explicit --config must exist, the default path may be missing
        let (path, required) = match args.config {
            Some(p) => (Some(p), true),
            None => (default_path(), false),
        };
        if let Some(path) = path {
            match fs::read_to_string(&path) {
                Ok(text) => {
                    let file: FileConfig =
                        toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
                    config.apply_file(file);
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound && !required => {}
                Err(e) => return Err(format!("{}: {e}", path.display()).into()),
            }
        }

        // 2. CLI args win over the file
        if let Some(symbol) = args.symbol {
            config.symbol = symbol;
        }

        config.validate()?;
        Ok(config)
    }

    fn apply_file(&mut self, f: FileConfig) {
        if let Some(v) = f.symbol {
            self.symbol = v;
        }
        if let Some(v) = f.history_len {
            self.history_len = v;
        }
        if let Some(v) = f.reconnect_delay_secs {
            self.reconnect_delay = Duration::from_secs(v);
        }
        if let Some(v) = f.channel_capacity {
            self.channel_capacity = v;
        }
    }

    // Normalize the symbol and reject values that would panic or never work.
    fn validate(&mut self) -> Result<(), String> {
        self.symbol = normalize_symbol(&self.symbol)?;
        if self.history_len == 0 {
            return Err("history_len must be at least 1".into());
        }
        if self.channel_capacity == 0 {
            // tokio::sync::mpsc::channel(0) panics
            return Err("channel_capacity must be at least 1".into());
        }
        Ok(())
    }

    // Binance stream names are lowercase: wss://.../ws/ethusdc@ticker
    pub fn ws_url(&self) -> String {
        format!(
            "wss://stream.binance.com:9443/ws/{}@ticker",
            self.symbol.to_lowercase()
        )
    }
}

// $XDG_CONFIG_HOME/talos/config.toml, or ~/.config/talos/config.toml
fn default_path() -> Option<PathBuf> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("talos").join("config.toml"))
}

fn normalize_symbol(s: &str) -> Result<String, String> {
    let s = s.trim().to_uppercase();
    if s.is_empty() || !s.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!("invalid symbol: {s:?}"));
    }
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_is_uppercased_and_checked() {
        assert_eq!(normalize_symbol(" btcusdt ").unwrap(), "BTCUSDT");
        assert!(normalize_symbol("").is_err());
        assert!(normalize_symbol("ETH/USDC").is_err());
    }

    #[test]
    fn file_overrides_only_what_it_sets() {
        let file: FileConfig =
            toml::from_str("symbol = \"solusdc\"\nreconnect_delay_secs = 10").unwrap();
        let mut c = Config::default();
        c.apply_file(file);
        c.validate().unwrap();
        assert_eq!(c.symbol, "SOLUSDC");
        assert_eq!(c.reconnect_delay, Duration::from_secs(10));
        assert_eq!(c.history_len, 2000); // untouched
    }

    #[test]
    fn unknown_key_is_rejected() {
        assert!(toml::from_str::<FileConfig>("symbl = \"ETHUSDC\"").is_err());
    }
}
