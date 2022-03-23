pub mod connections;

mod anonymous_level;
pub use anonymous_level::AnonymousLevel;

use tonic::{Request, Response, Status};

tonic::include_proto!("orderbook");

#[derive(Debug)]
pub struct OrderbookAggregator;

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

pub struct SimpleOrderBook {
    pub bids: Vec<AnonymousLevel>,
    pub asks: Vec<AnonymousLevel>,
}
