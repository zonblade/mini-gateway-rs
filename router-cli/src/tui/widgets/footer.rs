use crate::tui::app::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn draw(f: &mut Frame, area: Rect, app: &AppState) {
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
