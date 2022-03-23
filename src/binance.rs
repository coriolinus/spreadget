use crate::{AnonymousLevel, ExchangeConnection, SimpleOrderBook};
use futures::{StreamExt, TryStreamExt};
use serde::{
    de::{Error as _, SeqAccess},
    Deserialize,
};

/// This helper type exists so that we can deserialize a string representation of a number into itself.
struct StringFloat(f64);

impl<'de> Deserialize<'de> for StringFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = StringFloat;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("string containing a floating point literal")
            }

            /// In case we get a raw float instead of a string, we can handle it.
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                Ok(StringFloat(v))
            }

            /// In case we get a string, try to parse it as a float.
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse::<f64>()
                    .map(StringFloat)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &self))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl From<StringFloat> for f64 {
    fn from(sf: StringFloat) -> Self {
        sf.0
    }
}

/// This helper type exists so that we can deserialize a list of a pair of strings as numbers.
///
/// Binance uses an odd format for its bids and asks: `bids: [["1.0", "2.0"], ["3.4", "5.6"]]`.
/// This lets us deserialize that format.
#[derive(Debug)]
struct PriceAndQty {
    price: f64,
    qty: f64,
}

impl<'de> Deserialize<'de> for PriceAndQty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PriceAndQty;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("list of two numbers")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut expect_element = |field_name: &'static str| {
                    seq.next_element::<StringFloat>()?
                        .ok_or_else(|| A::Error::missing_field(field_name))
                };

                let price = expect_element("price")?;
                let qty = expect_element("qty")?;

                if seq.next_element::<StringFloat>()?.is_some() {
                    return Err(A::Error::invalid_length(3, &self));
                }

                Ok(PriceAndQty {
                    price: price.into(),
                    qty: qty.into(),
                })
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

/// Message type for Binance partial book stream.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookDepthStreamMessage {
    _last_update_id: usize,
    bids: Vec<PriceAndQty>,
    asks: Vec<PriceAndQty>,
}

impl From<PriceAndQty> for AnonymousLevel {
    fn from(PriceAndQty { price, qty: amount }: PriceAndQty) -> Self {
        AnonymousLevel { price, amount }
    }
}

impl From<BookDepthStreamMessage> for SimpleOrderBook {
    fn from(msg: BookDepthStreamMessage) -> Self {
        SimpleOrderBook {
            bids: msg.bids.into_iter().map(Into::into).collect(),
            asks: msg.asks.into_iter().map(Into::into).collect(),
        }
    }
}

/// Manage a websocket connection to Binance.
struct BinanceConnection;

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
                    let message = maybe_message?.into_text()?;
                    let bdsm: BookDepthStreamMessage = serde_json::from_str(&message)?;
                    Ok(bdsm.into())
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
