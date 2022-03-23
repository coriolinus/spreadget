pub mod binance;
pub mod bitstamp;

use crate::SimpleOrderBook;

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
