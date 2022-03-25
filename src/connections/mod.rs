pub mod binance;
pub mod bitstamp;

use crate::SimpleOrderBook;
use serde::de::DeserializeOwned;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::{
    error::Error as TungsteniteError, Message as TungsteniteMessage,
};

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
        symbol: String,
        updates: Sender<(&'static str, SimpleOrderBook)>,
    ) -> Result<(), Box<dyn 'static + std::error::Error + Send>>;
}

/// Convert a potential tungstenite message into a `SimpleOrderBook`.
///
/// This function mainly exists to simplify the error-handling story.
///
/// Certain messages are irrelevant and will be marked as such with the
/// [`Error::irrelevant`] method. These messages should be discarded by
/// the caller without breaking the message loop.
pub(crate) fn read_book<Message, Err>(
    event: Result<tokio_tungstenite::tungstenite::Message, TungsteniteError>,
) -> Result<SimpleOrderBook, Err>
where
    Message: Into<SimpleOrderBook> + DeserializeOwned,
    Err: Error + From<TungsteniteError> + From<serde_json::Error>,
{
    let message = event?;
    if let TungsteniteMessage::Ping(_) | TungsteniteMessage::Pong(_) = &message {
        return Err(Error::irrelevant());
    }
    log::debug!("{message:?}");
    let message: Message = serde_json::from_str(&message.into_text()?)?;
    Ok(message.into())
}

pub trait Error {
    /// Notify that this particular message can safely be ignored.
    ///
    /// This is true most often for `ping` messages, which are automatically responded to by
    /// the underlying tungstenite library.
    fn irrelevant() -> Self;
}
