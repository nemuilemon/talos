use std::collections::VecDeque;
use std::error::Error;

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures_util::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::ticker::Tick;
use crate::ui;
use crate::ws::{Status, WsEvent};

// Screen state
pub struct App {
    pub config: Config,
    pub status: Status,
    pub latest: Option<Tick>,
    pub prev_last: Option<f64>,
    pub history: VecDeque<f64>,
}

impl App {
    fn new(config: Config) -> Self {
        App {
            status: Status::Connecting,
            latest: None,
            prev_last: None,
            history: VecDeque::with_capacity(config.history_len),
            config,
        }
    }

    fn on_ws(&mut self, ev: WsEvent) {
        match ev {
            WsEvent::Status(s) => self.status = s,
            WsEvent::Tick(t) => {
                self.prev_last = self.latest.as_ref().map(|old| old.last);
                if self.history.len() == self.config.history_len {
                    self.history.pop_front();
                }
                self.history.push_back(t.last);
                self.latest = Some(t);
            }
        }
    }
}

pub async fn run(
    terminal: &mut DefaultTerminal,
    rx: &mut mpsc::Receiver<WsEvent>,
    config: Config,
) -> Result<(), Box<dyn Error>> {
    let mut app = App::new(config);
    let mut keys = EventStream::new();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Wait for whichever comes first: a WebSocket message or a key press
        tokio::select! {
            Some(ev) = rx.recv() => app.on_ws(ev),
            Some(Ok(ev)) = keys.next() => {
                if let Event::Key(key) = ev
                    && key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                {
                    return Ok(());
                }
            }
            else => return Ok(()),
        }
    }
}
