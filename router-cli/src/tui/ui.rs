use crate::tui::app::{format_bytes, format_bytes_rate, format_count, AppState, Panel, Rates};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Bar, BarChart, BarGroup, Block, Borders, Clear, Padding, Paragraph, Sparkline, Wrap,
};
use ratatui::Frame;

const INTERVAL_SECS: f64 = 15.0;

pub fn draw(f: &mut Frame, app: &AppState) {
    let area = f.area();

    draw_main(f, area, app);

    if app.show_help {
        draw_help_overlay(f, area);
    }
}

fn draw_main(f: &mut Frame, area: Rect, app: &AppState) {
    let tall = area.height >= 30;
    let mid = area.height >= 20 && !tall;

    let constraints = if tall {
        vec![
            Constraint::Length(1), // header
            Constraint::Length(3), // overview (aggregate totals + error rate)
            Constraint::Length(6), // stats panels (rates)
            Constraint::Min(3),    // sparkline: requests
            Constraint::Min(3),    // sparkline: bytes_in
            Constraint::Min(3),    // sparkline: bytes_out
            Constraint::Length(5), // status codes bar chart
            Constraint::Length(1), // footer
        ]
    } else if mid {
        vec![
            Constraint::Length(1), // header
            Constraint::Length(3), // overview
            Constraint::Length(6), // stats panels
            Constraint::Length(5), // status codes
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

    let wide = area.width >= 100;
    let mut idx = 0;

    draw_header(f, chunks[idx]);
    idx += 1;

    if tall || mid {
        draw_overview(f, chunks[idx], app, wide);
        idx += 1;
    }

    if wide {
        draw_stats_side_by_side(f, chunks[idx], app);
    } else {
        draw_stats_stacked(f, chunks[idx], app);
    }
    idx += 1;

    if tall {
        draw_sparkline_line(f, chunks[idx], app, SparkKind::Requests);
        idx += 1;
        draw_sparkline_line(f, chunks[idx], app, SparkKind::BytesIn);
        idx += 1;
        draw_sparkline_line(f, chunks[idx], app, SparkKind::BytesOut);
        idx += 1;
    }

    draw_status_codes(f, chunks[idx], app);
    idx += 1;

    draw_footer(f, chunks[idx], app);
}

// -- Header --

fn draw_header(f: &mut Frame, area: Rect) {
    let header = Line::from(vec![
        Span::styled(
            " Mini-Gateway Monitor ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "q:quit  tab:switch  1:gw  2:px  ?:help",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    f.render_widget(Paragraph::new(header), area);
}

// -- Overview (aggregate totals) --

fn draw_overview(f: &mut Frame, area: Rect, app: &AppState, wide: bool) {
    let block = Block::default()
        .title(" Overview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let gw = app.aggregate_totals(Panel::Gateway);
    let px = app.aggregate_totals(Panel::Proxy);

    let dim = Style::default().fg(Color::DarkGray);
    let line = if wide {
        Line::from(vec![
            Span::raw("  GW "),
            success_badge(gw.success_rate),
            Span::raw(format!(
                " req:{} res:{} fail:{} err:{:.1}% in:{} out:{}",
                format_count(gw.total_req),
                format_count(gw.total_res),
                format_count(gw.total_failed),
                gw.error_rate,
                format_bytes(gw.total_bytes_in),
                format_bytes(gw.total_bytes_out),
            )),
            Span::styled(format!(" ({} intervals)", gw.intervals), dim),
            Span::raw("  PX "),
            success_badge(px.success_rate),
            Span::raw(format!(
                " req:{} res:{} fail:{} err:{:.1}%",
                format_count(px.total_req),
                format_count(px.total_res),
                format_count(px.total_failed),
                px.error_rate,
            )),
        ])
    } else {
        let focused = app.aggregate_totals(app.focus);
        let label = match app.focus {
            Panel::Gateway => "GW",
            Panel::Proxy => "PX",
        };
        Line::from(vec![
            Span::raw(format!("  {label} ")),
            success_badge(focused.success_rate),
            Span::raw(format!(
                " req:{} res:{} fail:{} err:{:.1}% in:{} out:{}",
                format_count(focused.total_req),
                format_count(focused.total_res),
                format_count(focused.total_failed),
                focused.error_rate,
                format_bytes(focused.total_bytes_in),
                format_bytes(focused.total_bytes_out),
            )),
            Span::styled(format!(" ({} intervals)", focused.intervals), dim),
        ])
    };

    f.render_widget(Paragraph::new(line).block(block), area);
}

fn success_badge(rate: f64) -> Span<'static> {
    let (color, bg) = if rate >= 99.0 {
        (Color::White, Color::Green)
    } else if rate >= 95.0 {
        (Color::Black, Color::Yellow)
    } else {
        (Color::White, Color::Red)
    };
    Span::styled(format!(" {rate:.1}% "), Style::default().fg(color).bg(bg))
}

// -- Stats Panels (rates) --

fn draw_stats_side_by_side(f: &mut Frame, area: Rect, app: &AppState) {
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_stat_panel(f, panels[0], app, Panel::Gateway);
    draw_stat_panel(f, panels[1], app, Panel::Proxy);
}

fn draw_stats_stacked(f: &mut Frame, area: Rect, app: &AppState) {
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

fn rate_lines(r: &Rates, t: &crate::tui::app::TargetStats) -> Vec<Line<'static>> {
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

// -- Sparklines --

enum SparkKind {
    Requests,
    BytesIn,
    BytesOut,
}

fn draw_sparkline_line(f: &mut Frame, area: Rect, app: &AppState, kind: SparkKind) {
    let panel = app.focus;
    let label = match panel {
        Panel::Gateway => "Gateway",
        Panel::Proxy => "Proxy",
    };

    let (suffix, color, data, rate_str) = match kind {
        SparkKind::Requests => {
            let data = app.req_series(panel);
            let rate = app
                .latest_rates(panel, INTERVAL_SECS)
                .map(|r| format!("{:.1} req/s", r.req_per_sec))
                .unwrap_or_default();
            ("requests", Color::Green, data, rate)
        }
        SparkKind::BytesIn => {
            let data = app.bytes_in_series(panel);
            let rate = app
                .latest_rates(panel, INTERVAL_SECS)
                .map(|r| format_bytes_rate(r.bytes_in_per_sec))
                .unwrap_or_default();
            ("bytes_in", Color::Cyan, data, rate)
        }
        SparkKind::BytesOut => {
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

// -- Status Codes --

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

// -- Footer --

fn draw_footer(f: &mut Frame, area: Rect, app: &AppState) {
    let (conn_text, conn_style) = if app.connected {
        ("Connected", Style::default().fg(Color::Green))
    } else {
        ("Disconnected", Style::default().fg(Color::Red))
    };

    let ts = app.latest().map(|s| s.ts.as_str()).unwrap_or("-");
    let points = app.history.len();
    let uptime = app.uptime_str();

    let footer = Line::from(vec![
        Span::raw(" "),
        Span::styled(conn_text, conn_style),
        Span::styled(
            format!("  Up: {uptime}  Last: {ts}  [{points}/120 pts]  "),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(&app.status_msg, Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(footer), area);
}

// -- Help Overlay --

fn draw_help_overlay(f: &mut Frame, area: Rect) {
    let width = 40u16.min(area.width.saturating_sub(4));
    let height = 14u16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    f.render_widget(Clear, popup);

    let help_text = vec![
        Line::from(Span::styled(
            "Keybindings",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  q / Ctrl+C    Quit"),
        Line::from("  Tab           Switch panel focus"),
        Line::from("  1             Focus Gateway"),
        Line::from("  2             Focus Proxy"),
        Line::from("  ?             Toggle this help"),
        Line::from(""),
        Line::from(Span::styled(
            "Info",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  SSE updates every 15s"),
        Line::from("  History: 120 pts (30 min)"),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(1));

    f.render_widget(
        Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: false }),
        popup,
    );
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
