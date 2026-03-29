use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use ratatui::Frame;

pub fn draw(f: &mut Frame, area: Rect) {
    let width = 40u16.min(area.width.saturating_sub(4));
    let height = 14u16.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    let popup = Rect::new(x, y, width, height);

    f.render_widget(Clear, popup);

    let bold_cyan = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let help_text = vec![
        Line::from(Span::styled("Keybindings", bold_cyan)),
        Line::from(""),
        Line::from("  q / Ctrl+C    Quit"),
        Line::from("  Tab           Switch panel focus"),
        Line::from("  1             Focus Gateway"),
        Line::from("  2             Focus Proxy"),
        Line::from("  ?             Toggle this help"),
        Line::from(""),
        Line::from(Span::styled("Info", bold_cyan)),
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
