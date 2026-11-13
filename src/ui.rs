use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Chart, Dataset, GraphType, Paragraph},
};

use crate::app::App;
use crate::ws::Status;

pub fn draw(frame: &mut Frame, app: &App) {
    let [header, chart, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    frame.render_widget(header_widget(app), header);
    // x = tick index, y = price
    let points: Vec<(f64, f64)> = app
        .history
        .iter()
        .enumerate()
        .map(|(i, &p)| (i as f64, p))
        .collect();
    frame.render_widget(chart_widget(app, &points), chart);

    let status = match &app.status {
        Status::Connecting => "connecting…".yellow(),
        Status::Live => "live".green(),
        Status::Reconnecting(reason) => format!(
            "reconnecting in {}s ({reason})",
            app.config.reconnect_delay.as_secs()
        )
        .red(),
    };
    let help = Line::from(vec![" q: quit  |  ".dark_gray(), status]);
    frame.render_widget(help, footer);
}

fn header_widget(app: &App) -> Paragraph<'_> {
    let block = Block::bordered().title(" Talos ");

    let Some(t) = &app.latest else {
        return Paragraph::new("waiting for the first tick…").block(block);
    };

    let (arrow, arrow_color) = match app.prev_last {
        None => (" ", Color::Reset),
        Some(p) if t.last > p => ("↑", Color::Green),
        Some(p) if t.last < p => ("↓", Color::Red),
        Some(_) => ("→", Color::Gray),
    };
    let pct_color = if t.pct >= 0.0 {
        Color::Green
    } else {
        Color::Red
    };

    let line = Line::from(vec![
        Span::raw(format!("{}  ", t.symbol)).bold(),
        Span::raw(format!("${:.2} ", t.last)).bold(),
        Span::styled(arrow, Style::new().fg(arrow_color)),
        Span::raw(format!("   bid ${:.2} / ask ${:.2}   ", t.bid, t.ask)),
        Span::styled(format!("24h {:+.2}%", t.pct), Style::new().fg(pct_color)),
        Span::raw(format!("   H ${:.2} / L ${:.2}", t.high, t.low)),
    ]);
    Paragraph::new(line).block(block)
}

// Chart only borrows the (x, y) slice, so the owner (the Vec) lives in the caller
fn chart_widget<'a>(app: &App, points: &'a [(f64, f64)]) -> Chart<'a> {
    let min = app.history.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = app
        .history
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    // With one value (or all equal) the range is 0, so add a little padding above and below
    let pad = ((max - min) * 0.1).max(0.5);
    let (y_lo, y_hi) = if app.history.is_empty() {
        (0.0, 1.0)
    } else {
        (min - pad, max + pad)
    };

    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::new().cyan())
        .data(points);

    Chart::new(vec![dataset])
        .block(Block::bordered().title(format!(" last {} ticks ", app.history.len())))
        .x_axis(Axis::default().bounds([0.0, app.config.history_len as f64]))
        .y_axis(
            Axis::default()
                .bounds([y_lo, y_hi])
                .labels([format!("{y_lo:.2}"), format!("{y_hi:.2}")]),
        )
}
