use crate::tui::app::{AppState, Panel, TargetStats};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Sparkline};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(7), // Stats panels
            Constraint::Min(5),    // Traffic sparklines
            Constraint::Length(3), // Status codes
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_stats_panels(f, chunks[1], app);
    draw_traffic(f, chunks[2], app);
    draw_status_codes(f, chunks[3], app);
    draw_footer(f, chunks[4], app);
}

fn draw_header(f: &mut Frame, area: Rect, _app: &AppState) {
    let header = Line::from(vec![
        Span::styled(
            " Mini-Gateway Monitor ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  q:quit  tab:switch  1:gateway  2:proxy"),
    ]);
    f.render_widget(Paragraph::new(header), area);
}

fn draw_stats_panels(f: &mut Frame, area: Rect, app: &AppState) {
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let (gw_style, px_style) = match app.focus {
        Panel::Gateway => (Color::Cyan, Color::White),
        Panel::Proxy => (Color::White, Color::Cyan),
    };

    draw_target_panel(f, panels[0], "GATEWAY", gw_style, app, Panel::Gateway);
    draw_target_panel(f, panels[1], "PROXY", px_style, app, Panel::Proxy);
}

fn draw_target_panel(
    f: &mut Frame,
    area: Rect,
    title: &str,
    border_color: Color,
    app: &AppState,
    panel: Panel,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let stats = app.latest().map(|s| match panel {
        Panel::Gateway => &s.gateway,
        Panel::Proxy => &s.proxy,
    });

    let lines = if let Some(s) = stats {
        stats_lines(s)
    } else {
        vec![Line::from("  Waiting for data...")]
    };

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn stats_lines(s: &TargetStats) -> Vec<Line<'static>> {
    vec![
        Line::from(format!(
            "  Req: {:<8} Res: {:<8} Failed: {}",
            s.req, s.res, s.failed
        )),
        Line::from(format!("  Stalled: {}", s.stalled_count)),
        Line::from(format!(
            "  In:  {} (min: {} max: {} avg: {})",
            format_bytes(s.bytes_in),
            format_bytes(s.bytes_in_min),
            format_bytes(s.bytes_in_max),
            format_bytes(s.bytes_in_avg as i64),
        )),
        Line::from(format!(
            "  Out: {} (min: {} max: {} avg: {})",
            format_bytes(s.bytes_out),
            format_bytes(s.bytes_out_min),
            format_bytes(s.bytes_out_max),
            format_bytes(s.bytes_out_avg as i64),
        )),
    ]
}

fn draw_traffic(f: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let panel = app.focus;
    let label = match panel {
        Panel::Gateway => "Gateway",
        Panel::Proxy => "Proxy",
    };

    let bytes_in = app.bytes_in_series(panel);
    let bytes_out = app.bytes_out_series(panel);

    let in_spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" {label} bytes_in "))
                .borders(Borders::ALL),
        )
        .data(&bytes_in)
        .style(Style::default().fg(Color::Green));

    let out_spark = Sparkline::default()
        .block(
            Block::default()
                .title(format!(" {label} bytes_out "))
                .borders(Borders::ALL),
        )
        .data(&bytes_out)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(in_spark, chunks[0]);
    f.render_widget(out_spark, chunks[1]);
}

fn draw_status_codes(f: &mut Frame, area: Rect, app: &AppState) {
    let codes = app.aggregate_status_codes(app.focus);

    let spans: Vec<Span> = codes
        .iter()
        .map(|(code, count)| {
            let color = match code.as_str() {
                c if c.starts_with('2') => Color::Green,
                c if c.starts_with('3') => Color::Blue,
                c if c.starts_with('4') => Color::Yellow,
                c if c.starts_with('5') => Color::Red,
                _ => Color::White,
            };
            Span::styled(format!(" {code}:{count} "), Style::default().fg(color))
        })
        .collect();

    let line = if spans.is_empty() {
        Line::from("  No status codes yet")
    } else {
        Line::from(spans)
    };

    let block = Block::default()
        .title(" Status Codes ")
        .borders(Borders::ALL);

    f.render_widget(Paragraph::new(line).block(block), area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &AppState) {
    let conn_style = if app.connected {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let conn_text = if app.connected {
        "Connected"
    } else {
        "Disconnected"
    };

    let ts = app.latest().map(|s| s.ts.as_str()).unwrap_or("-");

    let footer = Line::from(vec![
        Span::raw(" Status: "),
        Span::styled(conn_text, conn_style),
        Span::raw(format!("  Last: {ts}  ")),
        Span::styled(&app.status_msg, Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(footer), area);
}

fn format_bytes(b: i64) -> String {
    let abs = b.unsigned_abs();
    if abs >= 1_073_741_824 {
        format!("{:.1} GB", b as f64 / 1_073_741_824.0)
    } else if abs >= 1_048_576 {
        format!("{:.1} MB", b as f64 / 1_048_576.0)
    } else if abs >= 1_024 {
        format!("{:.1} KB", b as f64 / 1_024.0)
    } else {
        format!("{b} B")
    }
}
