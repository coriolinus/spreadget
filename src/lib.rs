use tonic::{Request, Response, Status};

tonic::include_proto!("orderbook");

#[derive(Debug)]
pub struct OrderbookAggregator {
    bids: Vec<Level>,
    asks: Vec<Level>,
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

pub trait ExchangeConnection {
    /// This is the name of the exchange
    ///
    /// It will be used to identify the exchange in the order book.
    const EXCHANGE_NAME: &'static str;
}
