#[cfg(feature = "tui")]
mod tui;

use spreadget::{
    connections::{binance::BinanceConnection, bitstamp::BitstampConnection, ExchangeConnection},
    OrderbookAggregator,
};
use std::{cell::RefCell, net::SocketAddr, sync::Arc};
use structopt::StructOpt;

#[cfg(feature = "tui")]
use tokio::{select, task::JoinError};

#[derive(Debug, StructOpt, Clone)]
struct Options {
    /// Market symbol to examine
    #[structopt(default_value = "ethbtc")]
    symbol: String,

    /// Address on which to serve gRPC streams of order books
    #[structopt(short, long, default_value = "[::1]:54321")]
    address: SocketAddr,

    /// Run a TUI dashboard instead of showing log output
    #[cfg(feature = "tui")]
    #[structopt(long)]
    tui: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    #[cfg(not(feature = "tui"))]
    env_logger::init();

    #[cfg(feature = "tui")]
    {
        if !options.tui {
            env_logger::init();
        }
    }

    let mut aggregator = OrderbookAggregator::new();
    aggregator.launch_grpc_service(options.address);
    let aggregator_future = aggregator.aggregate_orderbooks(
        &options.symbol,
        [
            Box::new(BinanceConnection) as Box<dyn 'static + ExchangeConnection + Send + Sync>,
            Box::new(BitstampConnection),
        ],
    );

    #[cfg(not(feature = "tui"))]
    aggregator_future.await;

    // Given the possibility of a TUI, we want the following behaviors:
    //
    // - The program exits cleanly when `aggregator_future` exits.
    // - If a TUI was enabled by flag, the program exits cleanly when the TUI exits.
    //
    // This is precisely what the `select` macro is for.
    #[cfg(feature = "tui")]
    select! {
        _ = aggregator_future => {
            // aggregator exited; TUI would stop updating if we continued
        }

        _ = if options.tui {
            let app = tui::App::from(options.clone());
            Box::pin(tui::launch(app)) as Pin<Box<dyn Future<Output=Result<Result<(), String>, JoinError>>>>
        } else { Box::pin(futures::future::pending()) } => {
            // tui exited; aggregator is not needed anymore
        }
    }

    Ok(())
}
