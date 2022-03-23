pub mod connections;

mod anonymous_level;
use std::time::Duration;

pub use anonymous_level::AnonymousLevel;

use connections::ExchangeConnection;
use futures::stream::FuturesUnordered;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

tonic::include_proto!("orderbook");

pub struct SimpleOrderBook {
    pub bids: Vec<AnonymousLevel>,
    pub asks: Vec<AnonymousLevel>,
}

#[derive(Debug, Default)]
pub struct OrderbookAggregator {
    summary: RwLock<Summary>,
}

impl OrderbookAggregator {
    /// Create an orderbook aggregator, setting some configuration values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin aggregating order books.
    ///
    /// This continuously updates a merged order book owned by `self`.
    pub fn aggregate_orderbooks(&mut self, connections: &[Box<dyn ExchangeConnection>]) {
        todo!()
    }
}

#[tonic::async_trait]
impl orderbook_aggregator_server::OrderbookAggregator for OrderbookAggregator {
    // TODO! we need a real type here
    type BookSummaryStream = futures::stream::Empty<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        todo!()
    }
}
