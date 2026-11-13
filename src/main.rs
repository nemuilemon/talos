mod app;
mod config;
mod ticker;
mod ui;
mod ws;

use std::error::Error;

use tokio::sync::mpsc;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::default();

    let (tx, mut rx) = mpsc::channel(config.channel_capacity);
    tokio::spawn(ws::ws_task(config.clone(), tx));

    let mut terminal = ratatui::init(); // raw mode + alternate screen
    let result = app::run(&mut terminal, &mut rx, config).await;
    ratatui::restore(); // always restore the terminal, even on error
    result
}
