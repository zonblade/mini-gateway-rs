use crate::tui::app::{format_bytes, format_count, AppState, Panel};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, area: Rect, app: &AppState, wide: bool) {
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
