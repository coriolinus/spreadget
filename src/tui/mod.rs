mod inputs;
mod ui;

use crossterm::event::{KeyCode, KeyEvent};
use std::{cell::RefCell, io::stdout, net::SocketAddr, sync::Arc, time::Duration};
use tokio::task::JoinHandle;
use tui::{backend::CrosstermBackend, Terminal};

use self::inputs::{Events, InputEvent};

/// This application drives a TUI display, pulling data from the gRPC exposed by the underlying service.
pub struct App {
    symbol: String,
    address: SocketAddr,
}

impl App {
    fn do_action(&mut self, key: KeyEvent) -> AppReturn {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            return AppReturn::Exit;
        }
        AppReturn::Nop
    }
}

impl From<super::Options> for App {
    fn from(options: super::Options) -> Self {
        Self {
            symbol: options.symbol,
            address: options.address,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppReturn {
    /// No action is required.
    Nop,
    /// Exit the application.
    Exit,
}

type SharedApp = App;

/// Launch the Text User Interface dashboard.
///
/// This spawns a new thread on which the new dashboard will run.
pub fn launch(app: SharedApp) -> JoinHandle<Result<(), String>> {
    tokio::task::spawn_blocking(|| tui(app).map_err(|err| err.to_string()))
}

fn tui(mut app: SharedApp) -> Result<(), Box<dyn std::error::Error>> {
    // configure crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);

    loop {
        // let app = app.borrow();
        // render
        terminal.draw(|rect| ui::draw(rect, &app))?;

        let result = match events.next()? {
            // process user input
            InputEvent::Input(key) => app.do_action(key),
            // handle absence of input
            InputEvent::Tick => todo!(),
        };

        if result == AppReturn::Exit {
            break;
        }
    }

    // restore terminal and clean up
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
