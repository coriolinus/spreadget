pub mod binance;
pub mod bitstamp;

use crate::SimpleOrderBook;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

#[tonic::async_trait]
pub trait ExchangeConnection {
    /// This is the name of the exchange
    ///
    /// It will be used to identify the exchange in the order book.
    fn exchange_name(&self) -> &'static str;

    /// Establish a websocket connection for the desired symbol, producing an async stream of order books
    /// for this connection.
    async fn connect(
        &self,
        symbol: &str,
        updates: Sender<(&'static str, SimpleOrderBook)>,
    ) -> Result<(), Box<dyn 'static + std::error::Error + Send>>;
}

/// Convert a potential tungstenite message into a `SimpleOrderBook`.
///
/// This function mainly exists to simplify the error-handling story.
pub(crate) fn read_book<Message, Error>(
    event: Result<tokio_tungstenite::tungstenite::Message, TungsteniteError>,
) -> Result<SimpleOrderBook, Error>
where
    Message: Into<SimpleOrderBook> + DeserializeOwned,
    Error: From<TungsteniteError> + From<serde_json::Error>,
{
    let message: Message = serde_json::from_str(&event?.into_text()?)?;
    Ok(message.into())
}
