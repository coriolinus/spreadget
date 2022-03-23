pub mod binance;
pub mod bitstamp;

use crate::SimpleOrderBook;

#[tonic::async_trait]
pub trait ExchangeConnection {
    /// This is the name of the exchange
    ///
    /// It will be used to identify the exchange in the order book.
    fn exchange_name(&self) -> String;

    /// `true` when the provided name matches the exchange's name.
    ///
    /// This is essentially just an optimization which avoids an allocation when all that's desired is equality comparison.
    fn name_equals(&self, name: &str) -> bool;

    /// Establish a websocket connection for the desired symbol, producing an async stream of order books
    /// for this connection.
    async fn connect(
        &self,
        symbol: &str,
    ) -> Result<
        Box<dyn futures::Stream<Item = Result<SimpleOrderBook, Box<dyn std::error::Error>>>>,
        Box<dyn std::error::Error>,
    >;
}
