# `spreadget`: Get the spread from two exchanges and publish as gRPC

- connect to two exchanges' websocket feeds simultaneously
- pull the current order books over those streaming connections for a given traded market, from each exchange
- merge and sort the order books to create a combined order book
- from the combined book, publish the spread (top 10 bids and asks) as gRPC stream

## Exchanges

### Binance

- Docs for Websocket connection: <https://github.com/binance-exchange/binance-official-api-docs/blob/master/web-socket-streams.md>
- Example API feed: <https://api.binance.com/api/v3/depth?symbol=ETHBTC>
- Websocket connection URL for Binance: wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms

### Bitstamp

- Docs: <https://www.bitstamp.net/websocket/v2/>
- Example API feed: <https://www.bitstamp.net/api/v2/order_book/ethbtc/>
- Example Websocket usage: <https://www.bitstamp.net/s/webapp/examples/order_book_v2.html>

## TODOS

- [ ] test suite!

## Logging

This program logs events of interest, as configured by [`env_logger`](https://docs.rs/env_logger/latest/env_logger/). See that documentation
for examples of how to configure the log filters. For basic use sufficient to see incoming order books, use

```text
$ RUST_LOG=info cargo run --
...
[2022-03-24T20:40:56Z INFO  spreadget] aggregator received new order book data from binance
[2022-03-24T20:40:57Z INFO  spreadget] aggregator received new order book data from binance
[2022-03-24T20:40:57Z INFO  spreadget] aggregator received new order book data from binance
[2022-03-24T20:40:57Z INFO  spreadget] aggregator received new order book data from binance
[2022-03-24T20:40:57Z INFO  spreadget] aggregator received new order book data from bitstamp
[2022-03-24T20:40:57Z INFO  spreadget] aggregator received new order book data from binance
[2022-03-24T20:40:57Z INFO  spreadget] aggregator received new order book data from bitstamp
```

## Testing

The simplest way to observe the full system behavior is to use two terminals. In the first, we run `spreadget`:

```bash
RUST_LOG=info cargo run -- --address '0.0.0.0:54321'
```

In the second, we can use the [`grpcurl` tool](https://github.com/fullstorydev/grpcurl) to observe the output:

```bash
grpcurl -plaintext -import-path src -proto orderbook.proto 127.0.0.1:54321 orderbook.OrderbookAggregator/BookSummary
```

Note that `grpcurl` requires access to the `.proto` definition in order to function properly. If not running from within
the `spreadget` root directory, adjust the `-import-path` argument appropriately.
