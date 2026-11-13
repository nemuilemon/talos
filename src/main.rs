mod app;
mod ticker;
mod ui;
mod ws;

use std::error::Error;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, mut rx) = mpsc::channel(64);
    tokio::spawn(ws::ws_task(tx));

    let mut terminal = ratatui::init(); // raw mode + alternate screen
    let result = app::run(&mut terminal, &mut rx).await;
    ratatui::restore(); // always restore the terminal, even on error
    result
}
