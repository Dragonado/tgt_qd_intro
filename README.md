# WOO X Order Book Visualizer

## How to run

```zsh
cd woo-orderbook
cargo run
```

## Demo

![Live ETH-USDT perpetual order book](assets/part1.gif)

## Getting Data from Woox

Woox provides 2 APIs to read orderbook data:

1. Snapshot: 

- Receive entire orderbook.
- Simple GET request
- Latency of ~200ms
- https://developer.woox.io/api-reference/endpoint/websocket/Orderbook_update#topic-orderbookupdate@symbol@depth


2. Subscription update:

- Receive updates of orderbook. Not entire orderbook.
- Websocket API
- Latency of ~50ms
- https://developer.woox.io/api-reference/endpoint/websocket/Orderbook_update#topic-orderbookupdate@symbol@depth 

There are 2 ingestion ideas that don't require any multi-threading. Both of them have performance issues.

### 1. Just read from orderbook snapshots

Pros: 

- Simpler update logic. (just replace current orderbook with new one).

Cons:

- Slower.
- Latency of ~200ms which is more than the subscription latency.

### 2. Read initially from snapshot and immediately update from subscription without buffering

We construct an orderbook initially from our snapshot. Then we keep querying updates from the subscription socket:

The logic is below:

```
while socket is active && we get an update:

- If update.previous_timestamp == order book timestamp, update orderbook with the new update and update timestamp.
- Else, missed an update in between. There is no way to recover so we read from a fresh snapshot.
```
Pros:

- No need for buffering because updates are not queued.
- Single threaded logic.
- Fast.
- update latency is ~50ms.

Cons:

- Very brittle. 
- A failed update will cause a cascade of failures where we continously read from snapshots (which is slow).
- This is like some form of thrashing, consider this case.

```
Update failed -> order book reads from snapshot -> read new update -> new update timestamp is already ahead of the snapshot because constructing snapshot is slow -> read from snapshot -> read new update -> read from snapshot -> etc
```

### MPSC buffer (chosen solution)

The solution is to queue the updates in a buffer and read from the buffer.

We have one thread dedicated to reading from the socket and add updates to a buffer.

We have the main thread that reads from this buffer and updates the orderbook.

Pros:

- Fast and robust.
- Latency is still ~50ms.
- std library only provides MPSC queue (would have used SPSC queue otherwise).
- Altough API supports multiple producers, for simplicity I chose a single one with bounded buffer that puts back pressure. 

Cons:

- Multi-threaded logic
- It maybe possible that we exceed buffer size (highly unlikely because producer is bottlenecked by network latency vs consumer is bottlenecked by IO latency)

## Why read 50 updates instead of just 5

The orderbook actually holds data for the best 50 bids/asks instead of the desired 5. 

This is because the subscrption socket API only allows a minimum depth of 50. 

For correctness purposes, snapshot depth should equal subscription depth. We cannot just maintain the best 5 bids/ask in our orderbook because a stale bid/ask can remain in our orderbook which will be wrong. 

So we maintain an orderbook of best 50 bids/asks and only display the best 5.

## Why choose Vec over BST

We have at most 50 elements in our data structure. The gains of using a data structure with contigious memory layout (Vec) beats tree-like data structures even though they have better time complexity.

With contigious memory we get this optimisation for free (provided by compiler and hardware): 

- SIMD
- prefetching
- good cache locality
- Instruction and memory level parallism.
- No pointer chasing or rebalancing like BST.

Note, this is just from intuition, I did not benchmark.
