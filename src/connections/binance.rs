//! Connection implementation for Binance.
//!
//! Binance uses a relatively simple protocol: we can register for the desired stream
//! and stream protocol by passing parameters to the endpoint, and then (as long as we
//! are registered only for a single stream) we know that all messages will be of a particular type.
//! The stream handler therefore just has to transform those messages appropriately.
//!
//! Example data:
//!
//! ```json
//! {"lastUpdateId":5071750763,"bids":[["0.07036500","13.01310000"],["0.07036400","0.10000000"],["0.07036200","1.45170000"],["0.07036000","0.26890000"],["0.07035800","0.15590000"],["0.07035700","1.02620000"],["0.07035600","0.10000000"],["0.07035400","9.44760000"],["0.07035200","0.10150000"],["0.07035000","0.10000000"],["0.07034900","0.62010000"],["0.07034800","0.10000000"],["0.07034600","0.10000000"],["0.07034500","0.01080000"],["0.07034400","0.35240000"],["0.07034300","3.12960000"],["0.07034200","0.12070000"],["0.07034100","0.05130000"],["0.07034000","4.79100000"],["0.07033800","0.10000000"]],"asks":[["0.07036600","6.77250000"],["0.07036700","0.90840000"],["0.07036800","9.41920000"],["0.07036900","1.12060000"],["0.07037000","2.64990000"],["0.07037100","1.26440000"],["0.07037200","0.12240000"],["0.07037300","2.58580000"],["0.07037400","4.24210000"],["0.07037500","0.04240000"],["0.07037600","1.43410000"],["0.07037700","2.96460000"],["0.07037800","0.31060000"],["0.07037900","0.11760000"],["0.07038000","15.76550000"],["0.07038100","12.05310000"],["0.07038200","0.20270000"],["0.07038300","0.12960000"],["0.07038400","1.88930000"],["0.07038500","0.14370000"]]}
//! {"lastUpdateId":5071750764,"bids":[["0.07036500","13.01310000"],["0.07036400","0.10000000"],["0.07036200","1.45170000"],["0.07036000","0.26890000"],["0.07035800","0.15590000"],["0.07035700","1.02620000"],["0.07035600","0.10000000"],["0.07035400","9.44760000"],["0.07035200","0.10150000"],["0.07035000","0.10000000"],["0.07034800","0.10000000"],["0.07034600","0.10000000"],["0.07034500","0.01080000"],["0.07034400","0.35240000"],["0.07034300","3.12960000"],["0.07034200","0.12070000"],["0.07034100","0.05130000"],["0.07034000","4.79100000"],["0.07033800","0.10000000"],["0.07033600","0.10000000"]],"asks":[["0.07036600","6.77250000"],["0.07036700","0.90840000"],["0.07036800","9.41920000"],["0.07036900","1.12060000"],["0.07037000","2.64990000"],["0.07037100","1.26440000"],["0.07037200","0.12240000"],["0.07037300","2.58580000"],["0.07037400","4.24210000"],["0.07037500","0.04240000"],["0.07037600","1.43410000"],["0.07037700","2.96460000"],["0.07037800","0.31060000"],["0.07037900","0.11760000"],["0.07038000","15.76550000"],["0.07038100","12.05310000"],["0.07038200","0.20270000"],["0.07038300","0.12960000"],["0.07038400","1.88930000"],["0.07038500","0.14370000"]]}
//! ```
//!
//! Note that the symbol for Bitcoin/Etherium is `ethbtc`; this is the reverse of what Bitstamp does.

use super::ExchangeConnection;
use crate::{AnonymousLevel, SimpleOrderBook};
use futures::{StreamExt, TryStreamExt};

/// Message type for Binance partial book stream.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    _last_update_id: usize,
    bids: Vec<AnonymousLevel>,
    asks: Vec<AnonymousLevel>,
}

impl From<Message> for SimpleOrderBook {
    fn from(msg: Message) -> Self {
        SimpleOrderBook {
            bids: msg.bids.into_iter().map(Into::into).collect(),
            asks: msg.asks.into_iter().map(Into::into).collect(),
        }
    }
}

/// Manage a websocket connection to Binance.
pub struct BinanceConnection;

#[tonic::async_trait]
impl ExchangeConnection for BinanceConnection {
    const EXCHANGE_NAME: &'static str = "binance";

    type Err = Error;

    async fn connect(
        symbol: &str,
    ) -> Result<Box<dyn futures::Stream<Item = Result<SimpleOrderBook, Self::Err>>>, Self::Err>
    {
        let endpoint = format!("wss://stream.binance.com:9443/ws/{symbol}@depth20@100ms");
        let (stream, _response) = tokio_tungstenite::connect_async(&endpoint).await?;

        Ok(Box::new(
            stream
                .map::<Result<SimpleOrderBook, Self::Err>, _>(|maybe_message| {
                    // just pass errors along
                    let message_text = maybe_message?.into_text()?;
                    let message: Message = serde_json::from_str(&message_text)?;
                    Ok(message.into())
                })
                .map_err(Into::into),
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("websocket problem")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::error::Error),
    #[error("failed to deserialize message")]
    Deserialization(#[from] serde_json::Error),
}
