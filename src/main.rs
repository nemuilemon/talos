mod app;
mod config;
mod ticker;
mod ui;
mod ws;

use std::error::Error;

use clap::Parser;
use tokio::sync::mpsc;

use crate::config::{Args, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = match Config::load(Args::parse()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("talos: {e}");
            std::process::exit(2);
        }
    };

    let (tx, mut rx) = mpsc::channel(config.channel_capacity);
    tokio::spawn(ws::ws_task(config.clone(), tx));

    let mut terminal = ratatui::init(); // raw mode + alternate screen
    let result = app::run(&mut terminal, &mut rx, config).await;
    ratatui::restore(); // always restore the terminal, even on error
    result
}
