use crate::client::ApiClient;
use crate::error::CliError;
use crate::tui::app::{AppState, Panel};
use crate::tui::event::{Event, EventHandler};
use crate::tui::ui;
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub async fn run(client: &mut ApiClient, base_url: &str) -> Result<(), CliError> {
    let token = client
        .token()
        .ok_or_else(|| CliError::Auth("not authenticated, cannot start monitor".into()))?;

    let sse_url = format!(
        "{}/api/v1/statistics/stream",
        base_url.trim_end_matches('/')
    );

    enable_raw_mode().map_err(CliError::Io)?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen).map_err(CliError::Io)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| CliError::Io(io::Error::other(e)))?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let result = run_app(&mut terminal, sse_url, token).await;

    disable_raw_mode().map_err(CliError::Io)?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(CliError::Io)?;
    terminal
        .show_cursor()
        .map_err(|e| CliError::Io(io::Error::other(e)))?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    sse_url: String,
    token: String,
) -> Result<(), CliError> {
    let mut app = AppState::new();
    let mut events = EventHandler::new(sse_url, token);

    loop {
        terminal
            .draw(|f| ui::draw(f, &app))
            .map_err(|e| CliError::Io(io::Error::other(e)))?;

        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.should_quit = true;
                    }
                    KeyCode::Tab => {
                        app.focus = match app.focus {
                            Panel::Gateway => Panel::Proxy,
                            Panel::Proxy => Panel::Gateway,
                        };
                    }
                    KeyCode::Char('1') => app.focus = Panel::Gateway,
                    KeyCode::Char('2') => app.focus = Panel::Proxy,
                    KeyCode::Char('?') => app.show_help = !app.show_help,
                    KeyCode::Esc => app.show_help = false,
                    _ => {}
                },
                Event::Key(_) => {} // Ignore Release/Repeat events
                Event::Tick => {}   // Redraw happens at top of loop
                Event::SseData(stats) => {
                    app.push_stats(*stats);
                    app.status_msg = String::new();
                }
                Event::SseBatch(batch) => {
                    let count = batch.len();
                    app.load_history(batch);
                    app.status_msg = format!("Loaded {count} history points");
                }
                Event::SseConnected => {
                    app.connected = true;
                    app.status_msg = "Connected".to_string();
                }
                Event::SseDisconnected(msg) => {
                    app.connected = false;
                    app.status_msg = format!("Disconnected: {msg}");
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
