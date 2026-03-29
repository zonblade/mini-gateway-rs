use crate::tui::app::{format_bytes, format_bytes_rate, AppState, Panel, Rates, TargetStats};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

const INTERVAL_SECS: f64 = 15.0;

pub fn draw_side_by_side(f: &mut Frame, area: Rect, app: &AppState) {
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_panel(f, panels[0], app, Panel::Gateway);
    draw_panel(f, panels[1], app, Panel::Proxy);
}

pub fn draw_stacked(f: &mut Frame, area: Rect, app: &AppState) {
    draw_panel(f, area, app, app.focus);
}

fn draw_panel(f: &mut Frame, area: Rect, app: &AppState, panel: Panel) {
    let title = match panel {
        Panel::Gateway => "GATEWAY",
        Panel::Proxy => "PROXY",
    };
    let border_color = if panel == app.focus {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let lines = match (
        app.latest_rates(panel, INTERVAL_SECS),
        app.latest_target(panel),
    ) {
        (Some(rates), Some(target)) => rate_lines(&rates, target),
        _ => vec![
            Line::from(""),
            Span::styled(
                "  Waiting for data...",
                Style::default().fg(Color::DarkGray),
            )
            .into(),
        ],
    };

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn rate_lines(r: &Rates, t: &TargetStats) -> Vec<Line<'static>> {
    let dim = Style::default().fg(Color::DarkGray);
    vec![
        Line::from(vec![
            Span::raw("  Req/s: "),
            Span::styled(
                format!("{:<8.1}", r.req_per_sec),
                Style::default().fg(Color::Green),
            ),
            Span::raw("Res/s: "),
            Span::styled(
                format!("{:<8.1}", r.res_per_sec),
                Style::default().fg(Color::Green),
            ),
            Span::raw("Failed: "),
            colored_count(r.failed, Color::Red, Color::Green),
            Span::raw("  Stalled: "),
            colored_count(r.stalled, Color::Yellow, Color::Green),
        ]),
        Line::from(vec![
            Span::raw("  In:  "),
            Span::styled(
                format!("{:<12}", format_bytes_rate(r.bytes_in_per_sec)),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!(
                    "min:{} max:{} avg:{}",
                    format_bytes(t.bytes_in_min),
                    format_bytes(t.bytes_in_max),
                    format_bytes(t.bytes_in_avg as i64),
                ),
                dim,
            ),
        ]),
        Line::from(vec![
            Span::raw("  Out: "),
            Span::styled(
                format!("{:<12}", format_bytes_rate(r.bytes_out_per_sec)),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(
                format!(
                    "min:{} max:{} avg:{}",
                    format_bytes(t.bytes_out_min),
                    format_bytes(t.bytes_out_max),
                    format_bytes(t.bytes_out_avg as i64),
                ),
                dim,
            ),
        ]),
    ]
}

fn colored_count(val: i64, bad_color: Color, good_color: Color) -> Span<'static> {
    let color = if val > 0 { bad_color } else { good_color };
    Span::styled(format!("{val}"), Style::default().fg(color))
}
