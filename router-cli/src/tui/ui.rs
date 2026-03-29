use crate::tui::app::AppState;
use crate::tui::widgets::{footer, header, help, overview, sparklines, stats, status_codes};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &AppState) {
    let area = f.area();
    let tall = area.height >= 30;
    let mid = area.height >= 20 && !tall;
    let wide = area.width >= 100;

    let constraints = if tall {
        vec![
            Constraint::Length(1), // header
            Constraint::Length(3), // overview
            Constraint::Length(6), // stats panels
            Constraint::Min(3),    // sparkline: requests
            Constraint::Min(3),    // sparkline: bytes_in
            Constraint::Min(3),    // sparkline: bytes_out
            Constraint::Length(5), // status codes
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

    let mut idx = 0;

    header::draw(f, chunks[idx]);
    idx += 1;

    if tall || mid {
        overview::draw(f, chunks[idx], app, wide);
        idx += 1;
    }

    if wide {
        stats::draw_side_by_side(f, chunks[idx], app);
    } else {
        stats::draw_stacked(f, chunks[idx], app);
    }
    idx += 1;

    if tall {
        sparklines::draw(f, chunks[idx], app, sparklines::Kind::Requests);
        idx += 1;
        sparklines::draw(f, chunks[idx], app, sparklines::Kind::BytesIn);
        idx += 1;
        sparklines::draw(f, chunks[idx], app, sparklines::Kind::BytesOut);
        idx += 1;
    }

    status_codes::draw(f, chunks[idx], app);
    idx += 1;

    footer::draw(f, chunks[idx], app);

    if app.show_help {
        help::draw(f, area);
    }
}
