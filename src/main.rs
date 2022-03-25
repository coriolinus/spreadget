use std::net::SocketAddr;

use spreadget::{
    connections::{binance::BinanceConnection, bitstamp::BitstampConnection, ExchangeConnection},
    OrderbookAggregator,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    /// Market symbol to examine
    #[structopt(default_value = "ethbtc")]
    symbol: String,

    /// Address on which to serve gRPC streams of order books
    #[structopt(short, long, default_value = "[::1]:54321")]
    address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let options = Options::from_args();

    let mut aggregator = OrderbookAggregator::new();
    aggregator.launch_grpc_service(options.address);
    aggregator
        .aggregate_orderbooks(
            &options.symbol,
            [
                Box::new(BinanceConnection) as Box<dyn 'static + ExchangeConnection + Send + Sync>,
                Box::new(BitstampConnection),
            ],
        )
        .await;

    Ok(())
}
