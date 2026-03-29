use crate::tui::app::AppState;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, area: Rect, app: &AppState) {
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

fn status_color(code: &str) -> Color {
    match code {
        c if c.starts_with('2') => Color::Green,
        c if c.starts_with('3') => Color::Blue,
        c if c.starts_with('4') => Color::Yellow,
        c if c.starts_with('5') => Color::Red,
        _ => Color::White,
    }
}
