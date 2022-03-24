use std::net::SocketAddr;

use spreadget::{
    connections::{binance::BinanceConnection, bitstamp::BitstampConnection, ExchangeConnection},
    orderbook_aggregator_server::OrderbookAggregatorServer,
    OrderbookAggregator,
};
use structopt::StructOpt;
use tonic::transport::Server;

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
    aggregator
        .aggregate_orderbooks(
            &options.symbol,
            [
                Box::new(BinanceConnection) as Box<dyn 'static + ExchangeConnection + Send + Sync>,
                Box::new(BitstampConnection),
            ],
        )
        .await;

    // TODO: this is wrong, we can't wait to start the gRPC server until after the aggregation has completed, of course
    let service = OrderbookAggregatorServer::new(aggregator);
    log::info!("Listening for gRPC connections on {}", options.address);
    Server::builder()
        .add_service(service)
        .serve(options.address)
        .await?;

    Ok(())
}
