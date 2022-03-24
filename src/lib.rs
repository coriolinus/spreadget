//! Get the best bids and asks from several exchanges, aggregate them, and select the best.
//!
//! The entry point for this module is [`OrderbookAggregator`].

pub mod connections;

mod anonymous_level;
pub use anonymous_level::AnonymousLevel;

use connections::ExchangeConnection;
use float_ord::FloatOrd;
use futures::Stream;
use std::task::Poll;
use tokio::sync::{mpsc, watch};
use tonic::{Request, Response, Status};

tonic::include_proto!("orderbook");

/// The instructions specify that the summary keeps track of only the best 10 bids/asks.
const SUMMARY_BID_ASK_LEN: usize = 10;

/// The simplest representation of an exchange's order book.
pub struct SimpleOrderBook {
    pub bids: Vec<AnonymousLevel>,
    pub asks: Vec<AnonymousLevel>,
}

/// Aggregate the order books of several exchanges into the best bids and asks from each combined.
#[derive(Debug, Default)]
pub struct OrderbookAggregator {
    summary: Summary,
    rx: Option<watch::Receiver<Summary>>,
}

impl OrderbookAggregator {
    /// Create an orderbook aggregator, setting some configuration values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin aggregating order books.
    ///
    /// This continuously updates a merged order book owned by `self`. It spawns a task per connection.
    ///
    /// `symbol` is the general market symbol we are interested in.
    pub async fn aggregate_orderbooks(
        &mut self,
        symbol: &str,
        connections: impl IntoIterator<Item = Box<dyn ExchangeConnection + Sync + Send>>,
    ) {
        let (orderbook_sender, mut orderbook_receiver) = mpsc::channel(16);
        let (summary_sender, summary_receiver) = watch::channel(self.summary.clone());
        self.rx = Some(summary_receiver);

        for connection in connections.into_iter() {
            let symbol = symbol.to_string();
            let sender = orderbook_sender.clone();
            // TODO: keep track of the join handles returned here, and reconnect on a `ConnectionTerminated` error
            tokio::spawn(async move { connection.connect(symbol, sender).await });
        }

        // now pull all the simple order books from the channel and merge them into the aggregate summary.
        while let Some((name, new_data)) = orderbook_receiver.recv().await {
            log::info!("aggregator received new order book data from {name}");

            // All this vector manipulation is relatively inefficient from a theoretical point of view,
            // but it's my contention that for the number of levels we actually have to keep track of,
            // we're gaining as much from keeping the cache as we're losing by using linear memory patterns.

            // for both bids and asks, we simply clear out the old data and add the new data.
            for (summary, new_data) in [
                (&mut self.summary.bids, new_data.bids),
                (&mut self.summary.asks, new_data.asks),
            ] {
                summary.retain(|level| level.exchange != name);
                summary.extend(
                    new_data
                        .into_iter()
                        .map(|anonymous_level| anonymous_level.associate(name.to_string())),
                );
            }

            // bids get reverse-sorted because the highest bid is the best
            self.summary
                .bids
                .sort_unstable_by_key(|level| std::cmp::Reverse(FloatOrd(level.amount)));
            self.summary
                .asks
                .sort_unstable_by_key(|level| FloatOrd(level.amount));

            if self.summary.bids.is_empty() || self.summary.asks.is_empty() {
                self.summary.spread = 0.0;
            } else {
                self.summary.spread = self.summary.asks[0].amount - self.summary.bids[0].amount;
            }

            for summary_list in [&mut self.summary.bids, &mut self.summary.asks] {
                summary_list.truncate(SUMMARY_BID_ASK_LEN);
            }

            // This technically returns a result, but we know it will never return an error because
            // `self.rx` ensures that there always exists at least one receiver.
            let _ = summary_sender.send(self.summary.clone());
        }

        panic!("if we get here all connections were lost and never reconnected");
    }
}

pub type SummaryResult = Result<Summary, Status>;

#[tonic::async_trait]
impl orderbook_aggregator_server::OrderbookAggregator for OrderbookAggregator {
    type BookSummaryStream = BookSummaryStream;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        if let Some(rx) = &self.rx {
            Ok(Response::new(BookSummaryStream { rx: rx.clone() }))
        } else {
            Err(Status::unavailable(
                "aggregator has not yet begun to aggregate",
            ))
        }
    }
}

/// Implementation of a stream of book summaries as demanded by the [`OrderbookAggregator`][`orderbook_aggregator_server::OrderbookAggregator`].
pub struct BookSummaryStream {
    rx: watch::Receiver<Summary>,
}

impl Stream for BookSummaryStream {
    type Item = SummaryResult;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.rx.has_changed() {
            // There is a new result.
            Ok(true) => {
                let result = self.rx.borrow_and_update().clone();
                Poll::Ready(Some(Ok(result)))
            }
            // There is not a new result.
            Ok(false) => Poll::Pending,
            // The sender has closed, probably; indicate that the receiver should disconnect.
            Err(_) => Poll::Ready(None),
        }
    }
}
