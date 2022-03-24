pub mod connections;

mod anonymous_level;
pub use anonymous_level::AnonymousLevel;

use connections::ExchangeConnection;
use float_ord::FloatOrd;
use tokio::sync::mpsc::channel;
use tonic::{Request, Response, Status};

tonic::include_proto!("orderbook");

/// The instructions specify that the summary keeps track of only the best 10 bids/asks.
const SUMMARY_BID_ASK_LEN: usize = 10;

pub struct SimpleOrderBook {
    pub bids: Vec<AnonymousLevel>,
    pub asks: Vec<AnonymousLevel>,
}

#[derive(Debug, Default)]
pub struct OrderbookAggregator {
    summary: Summary,
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
        symbol: &'static str,
        connections: impl IntoIterator<Item = Box<dyn ExchangeConnection + Sync + Send>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (sender, mut receiver) = channel(16);

        for connection in connections.into_iter() {
            let sender = sender.clone();
            // TODO: keep track of the join handles returned here, and reconnect on a `ConnectionTerminated` error
            tokio::spawn(async move { connection.connect(symbol, sender).await });
        }

        // now pull all the simple order books from the channel and merge them into the aggregate summary.
        while let Some((name, new_data)) = receiver.recv().await {
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

            todo!("now notify all grpc streams about the new orderbook data")
        }

        panic!("if we get here all connections were lost and never reconnected");
    }
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
