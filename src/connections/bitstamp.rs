//! Connection implementation for Bitstamp.
//!
//! The bitsteamp websocket endpoint requires a two-step connection protocol:
//! first connect to the general-purpose endpoint, then register for the desired
//! stream(s).
//!
//! This also means that we need to be able to handle, at least, messages whose
//! `data` field is empty.
//!
//! Example data:
//!
//! ```json
//! {"event":"bts:subscription_succeeded","channel":"order_book_ethbtc","data":{}}
//! {"data":{"timestamp":"1648041918","microtimestamp":"1648041918792209","bids":[["0.07010256","6.00000000"],["0.07008689","0.50000000"],["0.07008637","6.32274318"],["0.07007691","0.01531744"],["0.07007446","1.01185446"],["0.07007082","0.51407345"],["0.07006279","0.01514769"],["0.07006161","6.10000000"],["0.07005844","1.68586537"],["0.07005579","0.50000000"],["0.07005091","0.49978378"],["0.07004834","0.01513734"],["0.07004770","1.58000000"],["0.07003947","1.68592503"],["0.07002884","0.07953299"],["0.07002014","1.68650657"],["0.07002013","1.68642410"],["0.07001690","3.82000000"],["0.07000000","77.39020000"],["0.06998968","2.32992865"],["0.06998967","12.20000000"],["0.06998682","1.68590811"],["0.06990690","7.71000000"],["0.06987283","1.29680259"],["0.06987282","1.00000000"],["0.06986310","0.00289034"],["0.06984601","2.40889409"],["0.06984600","0.00289031"],["0.06982890","0.00289028"],["0.06981180","0.00289025"],["0.06979646","42.80000000"],["0.06979470","0.00289022"],["0.06978330","0.00289020"],["0.06977760","0.00289019"],["0.06976620","0.00290017"],["0.06976050","0.00290016"],["0.06974910","0.00290014"],["0.06974340","0.00290013"],["0.06973200","0.00290011"],["0.06972630","0.00290010"],["0.06971490","0.00290008"],["0.06970920","0.00290007"],["0.06969780","0.00290005"],["0.06969210","0.00290004"],["0.06968640","0.00290003"],["0.06968070","0.00290002"],["0.06967284","0.00291544"],["0.06966696","0.00291543"],["0.06966108","0.00291542"],["0.06965520","0.00291541"],["0.06964932","0.00291540"],["0.06964452","38.98000000"],["0.06964344","0.00291539"],["0.06964076","1.00000000"],["0.06963756","0.00291538"],["0.06963657","0.05004533"],["0.06963168","0.00291537"],["0.06962580","0.00291536"],["0.06961992","0.00291535"],["0.06961404","0.00291534"],["0.06960816","0.00291533"],["0.06960228","0.00291532"],["0.06959640","0.00291531"],["0.06959052","0.00291530"],["0.06958464","0.00291529"],["0.06957876","0.00291528"],["0.06957288","0.00291527"],["0.06956700","0.00291526"],["0.06956112","0.00291525"],["0.06955524","0.00291524"],["0.06954936","0.00291523"],["0.06954348","0.00291522"],["0.06953864","12.81225159"],["0.06953760","0.00291521"],["0.06953172","0.00292520"],["0.06952584","0.00114961"],["0.06951996","0.00292518"],["0.06951408","0.00292517"],["0.06950820","0.00292516"],["0.06950232","0.00292515"],["0.06950000","0.20258028"],["0.06949644","0.00292514"],["0.06949056","0.00292513"],["0.06948468","0.00292512"],["0.06947880","0.00292511"],["0.06947292","0.00292510"],["0.06946704","0.00292509"],["0.06946116","0.00292508"],["0.06945528","0.00292507"],["0.06944940","0.00292506"],["0.06944598","0.05018267"],["0.06944352","0.00292505"],["0.06943764","0.00292504"],["0.06943176","0.00292503"],["0.06942588","0.00292502"],["0.06942000","0.00292501"],["0.06941413","39.63691859"],["0.06941412","0.00292500"],["0.06940824","0.00292499"],["0.06940236","0.00292498"]],"asks":[["0.07015025","0.05000000"],["0.07015316","1.55000000"],["0.07015568","0.51323872"],["0.07015583","1.01162417"],["0.07016010","1.68642410"],["0.07016812","0.01536420"],["0.07016982","0.51070720"],["0.07017037","0.50000000"],["0.07018397","1.68588769"],["0.07018498","0.52158470"],["0.07018743","6.10000000"],["0.07018781","0.71315969"],["0.07020350","1.68599472"],["0.07020363","0.01490495"],["0.07020718","1.68652480"],["0.07021680","1.58000000"],["0.07022117","9.99004001"],["0.07022178","0.01559306"],["0.07023210","3.82000000"],["0.07024020","0.00289037"],["0.07025154","1.00000000"],["0.07025665","1.68563625"],["0.07025730","0.00289040"],["0.07027440","0.00289043"],["0.07027749","12.20000000"],["0.07029000","61.52120000"],["0.07029150","0.00289046"],["0.07030860","0.00289049"],["0.07032260","7.71000000"],["0.07032569","0.37035722"],["0.07032570","0.00289052"],["0.07034278","2.97712188"],["0.07034279","0.92605867"],["0.07034280","0.00289055"],["0.07035987","2.40771035"],["0.07035990","0.00289058"],["0.07037700","0.00289061"],["0.07039410","0.00288064"],["0.07040796","48.60000000"],["0.07041120","0.00288067"],["0.07042830","0.00288070"],["0.07043040","0.00289023"],["0.07044540","0.00288073"],["0.07044750","0.00289026"],["0.07045050","39.94000000"],["0.07046250","0.00288076"],["0.07046460","0.00289029"],["0.07047202","1.00000000"],["0.07047960","0.00288079"],["0.07048170","0.00289032"],["0.07049670","0.00288082"],["0.07049880","0.00289035"],["0.07051380","0.00288085"],["0.07051590","0.00289038"],["0.07053090","0.00288088"],["0.07053300","0.00289041"],["0.07054800","0.00288091"],["0.07055010","0.00289044"],["0.07056510","0.00288094"],["0.07056720","0.00289047"],["0.07056968","7.32613355"],["0.07058220","0.00288097"],["0.07058430","0.00289050"],["0.07059930","0.00288100"],["0.07060140","0.00289053"],["0.07061640","0.00288103"],["0.07061850","0.00289056"],["0.07063350","0.00287106"],["0.07063560","0.00289059"],["0.07065060","0.00287109"],["0.07065270","0.00288062"],["0.07066350","0.00290006"],["0.07066770","0.00287112"],["0.07066980","0.00288065"],["0.07068060","0.00290009"],["0.07068480","0.00287115"],["0.07068690","0.00288068"],["0.07069203","1.00000000"],["0.07069770","0.00290012"],["0.07070190","0.00287118"],["0.07070400","0.00288071"],["0.07071480","0.00290015"],["0.07071758","0.04990835"],["0.07071900","0.00287121"],["0.07072110","0.00288074"],["0.07073190","0.00290018"],["0.07073610","0.00287124"],["0.07073820","0.00288077"],["0.07074900","0.00289021"],["0.07075320","0.00287127"],["0.07075530","0.00288080"],["0.07076610","0.00289024"],["0.07077030","0.00287130"],["0.07077240","0.00288083"],["0.07078320","0.00289027"],["0.07078740","0.00287133"],["0.07078950","0.00288086"],["0.07080030","0.00289030"],["0.07080450","0.00287136"],["0.07080660","0.00288089"]]},"channel":"order_book_ethbtc","event":"data"}
//! {"data":{"timestamp":"1648041919","microtimestamp":"1648041919391460","bids":[["0.07010256","6.00000000"],["0.07008637","6.32274318"],["0.07007763","0.50000000"],["0.07007691","0.01531744"],["0.07007446","1.01185446"],["0.07007082","0.51407345"],["0.07006279","0.01514769"],["0.07006161","6.10000000"],["0.07005844","1.68586537"],["0.07005579","0.50000000"],["0.07005091","0.49978378"],["0.07004834","0.01513734"],["0.07004770","1.58000000"],["0.07003947","1.68592503"],["0.07002884","0.07953299"],["0.07002014","1.68650657"],["0.07002013","1.68642410"],["0.07001690","3.82000000"],["0.07000000","77.39020000"],["0.06998968","2.32992865"],["0.06998967","12.20000000"],["0.06998682","1.68590811"],["0.06990690","7.71000000"],["0.06987283","1.29680259"],["0.06987282","1.00000000"],["0.06986310","0.00289034"],["0.06984601","2.40889409"],["0.06984600","0.00289031"],["0.06982890","0.00289028"],["0.06981180","0.00289025"],["0.06979646","42.80000000"],["0.06979470","0.00289022"],["0.06978330","0.00289020"],["0.06977760","0.00289019"],["0.06976620","0.00290017"],["0.06976050","0.00290016"],["0.06974910","0.00290014"],["0.06974340","0.00290013"],["0.06973200","0.00290011"],["0.06972630","0.00290010"],["0.06971490","0.00290008"],["0.06970920","0.00290007"],["0.06969780","0.00290005"],["0.06969210","0.00290004"],["0.06968640","0.00290003"],["0.06968070","0.00290002"],["0.06967284","0.00291544"],["0.06966696","0.00291543"],["0.06966108","0.00291542"],["0.06965520","0.00291541"],["0.06964932","0.00291540"],["0.06964452","38.98000000"],["0.06964344","0.00291539"],["0.06964076","1.00000000"],["0.06963756","0.00291538"],["0.06963657","0.05004533"],["0.06963168","0.00291537"],["0.06962580","0.00291536"],["0.06961992","0.00291535"],["0.06961404","0.00291534"],["0.06960816","0.00291533"],["0.06960228","0.00291532"],["0.06959640","0.00291531"],["0.06959052","0.00291530"],["0.06958464","0.00291529"],["0.06957876","0.00291528"],["0.06957288","0.00291527"],["0.06956700","0.00291526"],["0.06956112","0.00291525"],["0.06955524","0.00291524"],["0.06954936","0.00291523"],["0.06954348","0.00291522"],["0.06953864","12.81225159"],["0.06953760","0.00291521"],["0.06953172","0.00292520"],["0.06952584","0.00114961"],["0.06951996","0.00292518"],["0.06951408","0.00292517"],["0.06950820","0.00292516"],["0.06950232","0.00292515"],["0.06950000","0.20258028"],["0.06949644","0.00292514"],["0.06949056","0.00292513"],["0.06948468","0.00292512"],["0.06947880","0.00292511"],["0.06947292","0.00292510"],["0.06946704","0.00292509"],["0.06946116","0.00292508"],["0.06945528","0.00292507"],["0.06944940","0.00292506"],["0.06944598","0.05018267"],["0.06944352","0.00292505"],["0.06943764","0.00292504"],["0.06943176","0.00292503"],["0.06942588","0.00292502"],["0.06942000","0.00292501"],["0.06941413","39.63691859"],["0.06941412","0.00292500"],["0.06940824","0.00292499"],["0.06940236","0.00292498"]],"asks":[["0.07015025","0.05000000"],["0.07015316","1.55000000"],["0.07015568","0.51323872"],["0.07015583","1.01162417"],["0.07016010","1.68642410"],["0.07016110","0.50000000"],["0.07016812","0.01536420"],["0.07016982","0.51070720"],["0.07018397","1.68588769"],["0.07018498","0.52158470"],["0.07018743","6.10000000"],["0.07018781","0.71315969"],["0.07020350","1.68599472"],["0.07020363","0.01490495"],["0.07020718","1.68652480"],["0.07021680","1.58000000"],["0.07022117","9.99004001"],["0.07022178","0.01559306"],["0.07023210","3.82000000"],["0.07024020","0.00289037"],["0.07025154","1.00000000"],["0.07025665","1.68563625"],["0.07025730","0.00289040"],["0.07027440","0.00289043"],["0.07027749","12.20000000"],["0.07029000","61.52120000"],["0.07029150","0.00289046"],["0.07030860","0.00289049"],["0.07032260","7.71000000"],["0.07032569","0.37035722"],["0.07032570","0.00289052"],["0.07034278","2.97712188"],["0.07034279","0.92605867"],["0.07034280","0.00289055"],["0.07035987","2.40771035"],["0.07035990","0.00289058"],["0.07037700","0.00289061"],["0.07039410","0.00288064"],["0.07040796","48.60000000"],["0.07041120","0.00288067"],["0.07042830","0.00288070"],["0.07043040","0.00289023"],["0.07044540","0.00288073"],["0.07044750","0.00289026"],["0.07045050","39.94000000"],["0.07046250","0.00288076"],["0.07046460","0.00289029"],["0.07047202","1.00000000"],["0.07047960","0.00288079"],["0.07048170","0.00289032"],["0.07049670","0.00288082"],["0.07049880","0.00289035"],["0.07051380","0.00288085"],["0.07051590","0.00289038"],["0.07053090","0.00288088"],["0.07053300","0.00289041"],["0.07054800","0.00288091"],["0.07055010","0.00289044"],["0.07056510","0.00288094"],["0.07056720","0.00289047"],["0.07056968","7.32613355"],["0.07058220","0.00288097"],["0.07058430","0.00289050"],["0.07059930","0.00288100"],["0.07060140","0.00289053"],["0.07061640","0.00288103"],["0.07061850","0.00289056"],["0.07063350","0.00287106"],["0.07063560","0.00289059"],["0.07065060","0.00287109"],["0.07065270","0.00288062"],["0.07066350","0.00290006"],["0.07066770","0.00287112"],["0.07066980","0.00288065"],["0.07068060","0.00290009"],["0.07068480","0.00287115"],["0.07068690","0.00288068"],["0.07069203","1.00000000"],["0.07069770","0.00290012"],["0.07070190","0.00287118"],["0.07070400","0.00288071"],["0.07071480","0.00290015"],["0.07071758","0.04990835"],["0.07071900","0.00287121"],["0.07072110","0.00288074"],["0.07073190","0.00290018"],["0.07073610","0.00287124"],["0.07073820","0.00288077"],["0.07074900","0.00289021"],["0.07075320","0.00287127"],["0.07075530","0.00288080"],["0.07076610","0.00289024"],["0.07077030","0.00287130"],["0.07077240","0.00288083"],["0.07078320","0.00289027"],["0.07078740","0.00287133"],["0.07078950","0.00288086"],["0.07080030","0.00289030"],["0.07080450","0.00287136"],["0.07080660","0.00288089"]]},"channel":"order_book_ethbtc","event":"data"}
//! ```

use super::{read_book, ExchangeConnection};
use crate::{AnonymousLevel, SimpleOrderBook};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::Sender;

const EXCHANGE_NAME: &'static str = "bitstamp";

/// Message type for Bitstamp order book stream.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Message {
    _event: Option<String>,
    _channel: Option<String>,
    data: Data,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Data {
    _timestamp: String,
    _microtimestamp: String,
    bids: Vec<AnonymousLevel>,
    asks: Vec<AnonymousLevel>,
}

impl From<Message> for SimpleOrderBook {
    fn from(bsm: Message) -> Self {
        let Data { bids, asks, .. } = bsm.data;
        SimpleOrderBook { bids, asks }
    }
}

/// Manage a websocket connection to Bitstamp.
pub struct BitstampConnection;

#[tonic::async_trait]
impl ExchangeConnection for BitstampConnection {
    fn exchange_name(&self) -> &'static str {
        EXCHANGE_NAME
    }

    async fn connect(
        &self,
        symbol: String,
        updates: Sender<(&'static str, SimpleOrderBook)>,
    ) -> Result<(), Box<dyn 'static + std::error::Error + Send>> {
        log::trace!("[{EXCHANGE_NAME}] entered `connect` for {symbol}");

        let endpoint = "wss://ws.bitstamp.net";
        let (mut stream, _response) = tokio_tungstenite::connect_async(endpoint)
            .await
            .map_err(into_box)?;
        let subscription_message = format!(
            "{{\"event\":\"bts:subscribe\",\"data\":{{\"channel\":\"order_book_{symbol}\"}}}}"
        );
        stream
            .send(tokio_tungstenite::tungstenite::Message::Text(
                subscription_message,
            ))
            .await
            .map_err(into_box)?;

        let confirmation_message = stream
            .next()
            .await
            .ok_or(Error::NoConfirmation)
            .map_err(into_box)?
            .map_err(into_box)?;
        let confirmation_text = confirmation_message.into_text().map_err(into_box)?;
        if !confirmation_text.contains("bts:subscription_succeeded") {
            return Err(into_box(Error::SubscriptionFailure(confirmation_text)));
        }

        while let Some(maybe_message) = stream.next().await {
            match read_book::<Message, Error>(maybe_message) {
                Ok(book) => {
                    if let Err(_send_err) = updates.send((EXCHANGE_NAME, book)).await {
                        log::warn!("[{EXCHANGE_NAME}] terminating due to send failure indicating receiver closed");
                        return Ok(());
                    }
                }
                Err(err) => {
                    log::error!("[{EXCHANGE_NAME}] terminating due to error: {err}");
                    return Err(Box::new(err));
                }
            }
        }

        log::warn!("[{EXCHANGE_NAME}] websocket connection terminated");
        Err(Box::new(Error::ConnectionDropped))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("server did not send a textual confirmation message of stream registration")]
    NoConfirmation,
    #[error("subscription failure: {0}")]
    SubscriptionFailure(String),
    #[error("websocket problem")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::error::Error),
    #[error("failed to deserialize message")]
    Deserialization(#[from] serde_json::Error),
    #[error("connection dropped by host")]
    ConnectionDropped,
}

fn into_box(err: impl Into<Error>) -> Box<dyn 'static + std::error::Error + Send> {
    Box::new(err.into()) as _
}
