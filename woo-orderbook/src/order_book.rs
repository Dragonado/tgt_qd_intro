use std::io::{self, Write};
use std::io::{Error, ErrorKind};

#[derive(Debug, Default)]
pub(crate) struct PriceLevel {
    pub(crate) price: String,
    pub(crate) quantity: String,
}

#[derive(Debug, Default)]
pub(crate) struct OrderBookUpdate {
    pub(crate) symbol: String,
    pub(crate) previous_timestamp: u64,
    pub(crate) timestamp: u64,
    pub(crate) asks: Vec<PriceLevel>,
    pub(crate) bids: Vec<PriceLevel>,
}

// Responsibilities of OrderBook:
// - Verify update.symbol == self.symbol
// - Verify update.prevTs == self.timestamp
// - Apply insert/update/delete rules
// - Advance timestamp
#[derive(Debug, Default)]
pub(crate) struct OrderBook {
    timestamp: u64,
    symbol: String,
    max_levels: usize,
    asks: Vec<PriceLevel>,
    bids: Vec<PriceLevel>,
}

impl OrderBook {
    pub(crate) fn new(symbol: &str, max_levels: usize) -> Self {
        Self {
            timestamp: 0,
            symbol: symbol.to_string(),
            max_levels,
            asks: Vec::new(),
            bids: Vec::new(),
        }
    }

    pub(crate) fn from_snapshot(
        symbol: &str,
        max_levels: usize,
        timestamp: u64,
        asks: Vec<PriceLevel>,
        bids: Vec<PriceLevel>,
    ) -> Self {
        // asks.sort();
        // bids.sort();
        Self {
            timestamp,
            symbol: symbol.to_string(),
            max_levels,
            asks,
            bids,
        }
    }

    // If order_book.previous_timestamp == order_book.timestamp then we incrementally update the order book.
    // Else we throw an error.
    // In the second case, we must have missed an update in between and hence need to recover from a snapshot.
    pub(crate) fn ingest_incremental_update(
        &mut self,
        update: OrderBookUpdate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // // Validate and apply update.
        // self.asks = update.asks;
        // self.bids = update.bids;
        // self.timestamp = update.timestamp;

        Err("Not taking updates rn".into())
    }

    pub(crate) fn print_state(&self) {
        print!("\x1B[H\x1B[J");

        println!(
            "{:>4} | {:>14} | {:>14} | {:>14} | {:>14}",
            "#", "SIZE", "BID", "ASK", "SIZE"
        );
        println!("{}", "-".repeat(72));

        for (index, (bid, ask)) in self
            .bids
            .iter()
            .rev()
            .take(self.max_levels)
            .zip(self.asks.iter().take(self.max_levels))
            .enumerate()
        {
            println!(
                "{:>4} | {:>14} | {:>14} | {:>14} | {:>14}",
                index + 1,
                bid.quantity,
                bid.price,
                ask.price,
                ask.quantity
            );
        }
        io::stdout().flush().unwrap();
    }
}
