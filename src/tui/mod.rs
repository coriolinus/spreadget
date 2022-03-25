mod app;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use spreadget::{orderbook_aggregator_client::OrderbookAggregatorClient, Empty};
use std::{io, time::Duration};
use tokio::select;
use tonic::transport::Endpoint;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use crate::{tui::app::App, Options};

pub(crate) async fn run(options: Options) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // create app and run it
    let app = App::new(options);
    let res = run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res.map_err(Into::into)
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    // create the event stream which captures keyboard/mouse events
    let mut event_stream = EventStream::new();

    // connect to the gRPC port which the other half of the system is providing
    // note that this assumes that that service is listening on a loopback address
    let endpoint = Endpoint::from_shared(format!("localhost:{}", app.options.address.port()))?
        .connect_timeout(Duration::from_secs(1));
    let mut client = None;
    for _ in 0..5 {
        client = OrderbookAggregatorClient::connect(endpoint.clone())
            .await
            .ok();
        if client.is_some() {
            break;
        }
    }
    let mut client = match client {
        Some(client) => client,
        None => anyhow::bail!("failed 5x to connect to local gRPC client ({endpoint:?})"),
    };
    let mut summary_stream = client.book_summary(Empty {}).await?.into_inner();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let event = event_stream.next().fuse();
        let summary = summary_stream.next().fuse();

        select! {
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        if [
                            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                            KeyCode::Esc.into(),
                            KeyCode::Char('q').into()
                        ].into_iter().map(Event::Key).any(|quit| quit == event) {
                            app.on_quit_key();
                        }}
                    Some(Err(err)) => log::error!("[event] {err}"),
                    None => break,
                }
            }
            maybe_summary = summary => {
                match maybe_summary {
                    Some(Ok(summary)) => {
                        app.on_new_summary(summary);
                    }
                    Some(Err(err)) => log::error!("[summary] {err}"),
                    None => break,
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
