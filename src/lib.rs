pub mod binance;

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

pub struct AnonymousLevel {
    pub price: f64,
    pub amount: f64,
}

pub struct SimpleOrderBook {
    pub bids: Vec<AnonymousLevel>,
    pub asks: Vec<AnonymousLevel>,
}

#[tonic::async_trait]
pub trait ExchangeConnection {
    /// This is the name of the exchange
    ///
    /// It will be used to identify the exchange in the order book.
    const EXCHANGE_NAME: &'static str;

    /// Error type for this connection.
    type Err: std::error::Error;

    /// Establish a websocket connection for the desired symbol, producing an async stream of order books
    /// for this connection.
    async fn connect(
        symbol: &str,
    ) -> Result<Box<dyn futures::Stream<Item = Result<SimpleOrderBook, Self::Err>>>, Self::Err>;
}
