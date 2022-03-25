//! Get the best bids and asks from several exchanges, aggregate them, and select the best.
//!
//! The entry point for this module is [`OrderbookAggregator`].

pub mod connections;

mod anonymous_level;
pub use anonymous_level::AnonymousLevel;

use connections::ExchangeConnection;
use float_ord::FloatOrd;
use futures::{stream::FuturesUnordered, Stream, StreamExt};
use orderbook_aggregator_server::OrderbookAggregatorServer;
use std::{net::SocketAddr, task::Poll};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
};
use tonic::{transport::Server, Request, Response, Status};

tonic::include_proto!("orderbook");

/// The instructions specify that the summary keeps track of only the best 10 bids/asks.
const SUMMARY_BID_ASK_LEN: usize = 10;

/// The simplest representation of an exchange's order book.
pub struct SimpleOrderBook {
    pub bids: Vec<AnonymousLevel>,
    pub asks: Vec<AnonymousLevel>,
}

/// Observe a collection of join handles.
///
/// When one joins with a task error, cancel all the others. When all have joined, send a message
/// on the `joined` channel.
///
/// Note that this does not take any particular action if a task concludes with an `Ok` result.
/// If a task exits in this way, other tasks will continue running.
async fn handle_join_handles(
    mut join_handles: FuturesUnordered<
        JoinHandle<Result<(), Box<dyn 'static + std::error::Error + Send>>>,
    >,
    joined: oneshot::Sender<()>,
) {
    log::trace!(
        "entered `handle_join_handles` with {} join handles",
        join_handles.len()
    );

    loop {
        match join_handles.next().await {
            None => {
                break;
            }
            Some(Err(join_error)) => {
                // almost every time, this triggers when a task was explicitly aborted
                log::debug!("task joined with join error: {join_error}");
            }
            Some(Ok(Err(task_error))) => {
                // let's get the whole error chain compacted into one message
                let mut error = &*task_error as &(dyn std::error::Error);
                let mut message =
                    format!("task joined successfully with an error result: {task_error}");
                while let Some(cause) = error.source() {
                    error = cause;
                    message += &format!(": {error}");
                }
                log::error!("{message}");

                // If we've exited with an error condition, clean up all the other tasks instead of letting
                // the system run with an incomplete set of connections. This forces a restart of the entire
                // system by some external user.
                for handle in join_handles.iter() {
                    handle.abort();
                }
            }
            Some(Ok(Ok(()))) => log::trace!("task joined successfully with successful result"),
        }
    }

    // if the send fails then there was no point sending this message anyway, so no issues.
    let _ = joined.send(());
    log::debug!("all join handles have been aborted");
}

/// Aggregate the order books of several exchanges into the best bids and asks from each combined.
///
/// In general, the order of operations will be to create the instance with `new`, launch a grpc service (if desired)
/// with `launch_grpc_service`, and then begin aggregation with `aggregate_orderbooks`.
#[derive(Debug)]
pub struct OrderbookAggregator {
    summary: Summary,
    summary_sender: watch::Sender<Summary>,
    summary_receiver: watch::Receiver<Summary>,
}

impl OrderbookAggregator {
    /// Create an orderbook aggregator.
    pub fn new() -> Self {
        let summary = Summary::default();
        let (summary_sender, summary_receiver) = watch::channel(summary.clone());
        Self {
            summary,
            summary_sender,
            summary_receiver,
        }
    }

    /// Spawn a new task listening on the specified address and serving gRPC requests, returning immediately.
    pub fn launch_grpc_service(&self, address: SocketAddr) {
        let summary_receiver = self.summary_receiver.clone();
        let service =
            OrderbookAggregatorServer::new(OrderbookAggregatorService { summary_receiver });
        tokio::spawn(async move {
            log::info!("Listening for gRPC connections on {}", address);
            Server::builder().add_service(service).serve(address).await
        });
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
        log::trace!("entered `aggregate_orderbooks` for {symbol}");

        let (orderbook_sender, mut orderbook_receiver) = mpsc::channel(16);

        let join_handles = FuturesUnordered::new();

        for connection in connections.into_iter() {
            let symbol = symbol.to_string();
            let sender = orderbook_sender.clone();

            join_handles.push(tokio::spawn(async move {
                connection.connect(symbol, sender).await
            }));
        }

        let mut is_shutting_down = false;
        let (joined_tx, mut joined_rx) = oneshot::channel();
        tokio::spawn(handle_join_handles(join_handles, joined_tx));

        // now pull all the simple order books from the channel and merge them into the aggregate summary.
        //
        // this is a loop-select-match construct instead of just `while let Some(...) = orderbook_receiver.recv().await` because
        // we need the join handle monitor to be able to notify us that it's time to shut down.
        loop {
            tokio::select! {
                // if the join handle monitor indicates that all channels have closed, then we can close the
                // orderbook receiver for a graceful shutdown.
                _ = &mut joined_rx, if !is_shutting_down => {
                    orderbook_receiver.close();
                    is_shutting_down = true;
                },
                // otherwise we're going to wait for the next orderbook
                maybe_orderbook = orderbook_receiver.recv() => {
                    match maybe_orderbook {
                        None => break,
                        Some((name, new_data)) => {
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
                            // both bids and asks sort primarily by price, but secondarily by larger quantity
                            self.summary.bids.sort_unstable_by(|left, right|
                                FloatOrd(left.price).cmp(&FloatOrd(right.price))
                                    .reverse()
                                    .then_with(|| FloatOrd(left.amount).cmp(&FloatOrd(right.amount)).reverse()));
                            self.summary.asks.sort_unstable_by(|left, right|
                                FloatOrd(left.price).cmp(&FloatOrd(right.price))
                                    .then_with(|| FloatOrd(left.amount).cmp(&FloatOrd(right.amount)).reverse()));

                            if self.summary.bids.is_empty() || self.summary.asks.is_empty() {
                                self.summary.spread = 0.0;
                            } else {
                                self.summary.spread = self.summary.asks[0].price - self.summary.bids[0].price;
                            }
                            log::debug!(
                                "computed new spread: {:.10} ({:?} - {:?})",
                                self.summary.spread,
                                self.summary.asks[0],
                                self.summary.bids[0],
                            );

                            for summary_list in [&mut self.summary.bids, &mut self.summary.asks] {
                                summary_list.truncate(SUMMARY_BID_ASK_LEN);
                            }

                            // This technically returns a result, but we know it will never return an error because
                            // `self.summary_receiver` ensures that there always exists at least one receiver.
                            self.summary_sender
                                .send(self.summary.clone())
                                .expect("there is always at least one receiver");
                        }
                    }
                }
            }
        }

        log::debug!("`aggregate_orderbooks` going down; no more orderbooks are coming in");
    }
}

pub type SummaryResult = Result<Summary, Status>;

/// This service can respond to gRPC requests for a book summary stream, and deliver appropriate updates to that stream.
#[derive(Debug, Clone)]
pub struct OrderbookAggregatorService {
    summary_receiver: watch::Receiver<Summary>,
}

#[tonic::async_trait]
impl orderbook_aggregator_server::OrderbookAggregator for OrderbookAggregatorService {
    // Ideally we wouldn't have to implement our own `BookSummaryStream`, but would use a standard `WatchStream` instead.
    // Unfortunately, we can't: `WatchStream` requires that its items are `Clone`, but `Status` isn't, for some reason.
    type BookSummaryStream = BookSummaryStream;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        Ok(Response::new(BookSummaryStream {
            rx: self.summary_receiver.clone(),
        }))
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
