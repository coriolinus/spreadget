# `spreadget`: Get the spread from two exchanges and publish as gRPC

- connect to two exchanges' websocket feeds simultaneously
- pull the current order books over those streaming connections for a given traded market, from each exchange
- merge and sort the order books to create a combined order book
- from the combined book, publish the spread (top 10 bids and asks) as gRPC stream

## CLI Interface

```
spreadget 0.1.0

USAGE:
    spreadget [FLAGS] [OPTIONS] [symbol]

FLAGS:
    -h, --help       Prints help information
        --tui        Run a TUI dashboard instead of showing log output
    -V, --version    Prints version information

OPTIONS:
    -a, --address <address>    Address on which to serve gRPC streams of order books [default: 0.0.0.0:54321]

ARGS:
    <symbol>    Market symbol to examine [default: ethbtc]
```

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

## TUI

When built with feature `ticker` (enabled by default), the executable gains a `--tui` flag. This flag, when set, enables a
dashboard which streams the most current summaries via gRPC.

![image](https://user-images.githubusercontent.com/7822926/160366547-41071f08-4215-4246-9f27-e1a593ca8dde.png)
