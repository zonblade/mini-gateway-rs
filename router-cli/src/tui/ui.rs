use crate::tui::app::{format_bytes_rate, AppState, Panel, Rates};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph, Sparkline};
use ratatui::Frame;

/// SSE broadcasts every 15 seconds
const INTERVAL_SECS: f64 = 15.0;

pub fn draw(f: &mut Frame, app: &AppState) {
    let area = f.area();
    let tall = area.height >= 25;
    let wide = area.width >= 100;

    // Vertical layout: header, stats, sparklines (if tall), status codes, footer
    let constraints = if tall {
        vec![
            Constraint::Length(1), // header
            Constraint::Length(6), // stats panels
            Constraint::Min(4),    // sparkline: bytes_in
            Constraint::Min(4),    // sparkline: bytes_out
            Constraint::Length(5), // status codes bar chart
            Constraint::Length(1), // footer
        ]
    } else {
        vec![
            Constraint::Length(1), // header
            Constraint::Length(6), // stats panels
            Constraint::Length(5), // status codes
            Constraint::Length(1), // footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    draw_header(f, chunks[0]);

    if wide {
        draw_stats_side_by_side(f, chunks[1], app);
    } else {
        draw_stats_stacked(f, chunks[1], app);
    }

    if tall {
        draw_sparkline(f, chunks[2], app, true);
        draw_sparkline(f, chunks[3], app, false);
        draw_status_codes(f, chunks[4], app);
        draw_footer(f, chunks[5], app);
    } else {
        draw_status_codes(f, chunks[2], app);
        draw_footer(f, chunks[3], app);
    }
}

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Line::from(vec![
        Span::styled(
            " Mini-Gateway Monitor ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "q:quit  tab:switch  1:gw  2:px",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    f.render_widget(Paragraph::new(header), area);
}

fn draw_stats_side_by_side(f: &mut Frame, area: Rect, app: &AppState) {
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_stat_panel(f, panels[0], app, Panel::Gateway);
    draw_stat_panel(f, panels[1], app, Panel::Proxy);
}

fn draw_stats_stacked(f: &mut Frame, area: Rect, app: &AppState) {
    // In narrow mode, show only the focused panel
    draw_stat_panel(f, area, app, app.focus);
}

fn draw_stat_panel(f: &mut Frame, area: Rect, app: &AppState, panel: Panel) {
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

    let lines = match app.latest_rates(panel, INTERVAL_SECS) {
        Some(rates) => rate_lines(&rates),
        None => vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Waiting for data...",
                Style::default().fg(Color::DarkGray),
            )),
        ],
    };

    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn rate_lines(r: &Rates) -> Vec<Line<'static>> {
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
            Span::styled(
                format!("{}", r.failed),
                if r.failed > 0 {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
            Span::raw("  Stalled: "),
            Span::styled(
                format!("{}", r.stalled),
                if r.stalled > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
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
                    format_bytes(r.bytes_in_min),
                    format_bytes(r.bytes_in_max),
                    format_bytes(r.bytes_in_avg as i64),
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
                    format_bytes(r.bytes_out_min),
                    format_bytes(r.bytes_out_max),
                    format_bytes(r.bytes_out_avg as i64),
                ),
                dim,
            ),
        ]),
    ]
}

fn format_bytes(b: i64) -> String {
    let abs = b.unsigned_abs();
    if abs >= 1_073_741_824 {
        format!("{:.1}GB", b as f64 / 1_073_741_824.0)
    } else if abs >= 1_048_576 {
        format!("{:.1}MB", b as f64 / 1_048_576.0)
    } else if abs >= 1_024 {
        format!("{:.1}KB", b as f64 / 1_024.0)
    } else {
        format!("{b}B")
    }
}

fn draw_sparkline(f: &mut Frame, area: Rect, app: &AppState, is_bytes_in: bool) {
    let panel = app.focus;
    let label = match panel {
        Panel::Gateway => "Gateway",
        Panel::Proxy => "Proxy",
    };
    let (direction, color, data) = if is_bytes_in {
        ("bytes_in", Color::Cyan, app.bytes_in_series(panel))
    } else {
        ("bytes_out", Color::Magenta, app.bytes_out_series(panel))
    };

    // Show current rate in title
    let current_rate = app
        .latest_rates(panel, INTERVAL_SECS)
        .map(|r| {
            if is_bytes_in {
                format_bytes_rate(r.bytes_in_per_sec)
            } else {
                format_bytes_rate(r.bytes_out_per_sec)
            }
        })
        .unwrap_or_default();

    let title = format!(" {label} {direction}  {current_rate} ");

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

fn draw_status_codes(f: &mut Frame, area: Rect, app: &AppState) {
    let codes = app.aggregate_status_codes(app.focus);

    if codes.is_empty() {
        let block = Block::default()
            .title(" Status Codes ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let p = Paragraph::new(Line::from(Span::styled(
            "  No status codes yet",
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        f.render_widget(p, area);
        return;
    }

    let bars: Vec<Bar> = codes
        .iter()
        .map(|(code, count)| {
            let color = status_color(code);
            Bar::default()
                .value(*count as u64)
                .label(Line::from(code.clone()))
                .style(Style::default().fg(color))
                .value_style(Style::default().fg(Color::White).bg(color))
        })
        .collect();

    let chart = BarChart::default()
        .block(
            Block::default()
                .title(" Status Codes ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .data(BarGroup::default().bars(&bars))
        .bar_width(
            ((area.width as usize).saturating_sub(4))
                .checked_div(codes.len().max(1))
                .unwrap_or(3)
                .clamp(3, 12) as u16,
        )
        .bar_gap(1);

    f.render_widget(chart, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &AppState) {
    let (conn_text, conn_style) = if app.connected {
        ("Connected", Style::default().fg(Color::Green))
    } else {
        ("Disconnected", Style::default().fg(Color::Red))
    };

    let ts = app.latest().map(|s| s.ts.as_str()).unwrap_or("-");
    let points = app.history.len();

    let footer = Line::from(vec![
        Span::raw(" "),
        Span::styled(conn_text, conn_style),
        Span::styled(
            format!("  Last: {ts}  "),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!("[{points} pts] "),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(&app.status_msg, Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(footer), area);
}

fn status_color(code: &str) -> Color {
    match code {
        c if c.starts_with('2') => Color::Green,
        c if c.starts_with('3') => Color::Blue,
        c if c.starts_with('4') => Color::Yellow,
        c if c.starts_with('5') => Color::Red,
        _ => Color::White,
    }
}
