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
for examples of how to configure the log filters.
