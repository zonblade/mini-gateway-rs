use crate::tui::app::{format_bytes_rate, AppState, Panel};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Sparkline};
use ratatui::Frame;

const INTERVAL_SECS: f64 = 15.0;

pub enum Kind {
    Requests,
    BytesIn,
    BytesOut,
}

pub fn draw(f: &mut Frame, area: Rect, app: &AppState, kind: Kind) {
    let panel = app.focus;
    let label = match panel {
        Panel::Gateway => "Gateway",
        Panel::Proxy => "Proxy",
    };

    let (suffix, color, data, rate_str) = match kind {
        Kind::Requests => {
            let data = app.req_series(panel);
            let rate = app
                .latest_rates(panel, INTERVAL_SECS)
                .map(|r| format!("{:.1} req/s", r.req_per_sec))
                .unwrap_or_default();
            ("requests", Color::Green, data, rate)
        }
        Kind::BytesIn => {
            let data = app.bytes_in_series(panel);
            let rate = app
                .latest_rates(panel, INTERVAL_SECS)
                .map(|r| format_bytes_rate(r.bytes_in_per_sec))
                .unwrap_or_default();
            ("bytes_in", Color::Cyan, data, rate)
        }
        Kind::BytesOut => {
            let data = app.bytes_out_series(panel);
            let rate = app
                .latest_rates(panel, INTERVAL_SECS)
                .map(|r| format_bytes_rate(r.bytes_out_per_sec))
                .unwrap_or_default();
            ("bytes_out", Color::Magenta, data, rate)
        }
    };

    let title = format!(" {label} {suffix}  {rate_str} ");

    let spark = Sparkline::default()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .data(&data)
        .style(Style::default().fg(color));

    f.render_widget(spark, area);
}
