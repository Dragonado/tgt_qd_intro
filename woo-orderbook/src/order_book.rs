use info_derive::Info;
use std::io::{self, Write};

use rust_decimal::Decimal;

pub trait Info {
    fn info(&self);
}

#[derive(Debug, Default, Info)]
pub(crate) struct PriceLevel {
    pub(crate) price: Decimal,
    pub(crate) quantity: Decimal,
}

#[derive(Debug, Default, Info)]
pub(crate) struct OrderBookUpdate {
    pub(crate) symbol: String,
    pub(crate) previous_timestamp: u64,
    pub(crate) timestamp: u64,
    pub(crate) asks: Vec<PriceLevel>,
    pub(crate) bids: Vec<PriceLevel>,
}

#[derive(Debug, Default, Info)]
pub(crate) struct OrderBook {
    timestamp: u64,
    symbol: String,
    max_levels: usize,
    asks: Vec<PriceLevel>,
    bids: Vec<PriceLevel>,
}

impl OrderBook {
    // Construct an empty orderbook with the desired symbol and max_size.
    // Although not asserted, we expect a max_size of >= 50 or else orderbook might be holding wrong data.
    pub(crate) fn new(symbol: String, max_levels: usize) -> Self {
        Self {
            timestamp: 0,
            symbol,
            max_levels,
            asks: Vec::new(),
            bids: Vec::new(),
        }
    }

    // Construct an orderbook from snapshot of another orderbook.
    pub(crate) fn from_snapshot(
        symbol: String,
        max_levels: usize,
        timestamp: u64,
        mut asks: Vec<PriceLevel>,
        mut bids: Vec<PriceLevel>,
    ) -> Self {
        Self::filter_and_sort(&mut asks, max_levels, false);
        Self::filter_and_sort(&mut bids, max_levels, true);

        Self {
            timestamp,
            symbol,
            max_levels,
            asks,
            bids,
        }
    }

    // If order_book.previous_timestamp <= order_book.timestamp then we incrementally update (or skip the update) the order book.
    // Else, we throw an error.
    // In the second case, we must have missed an update in between and hence need to recover from a snapshot.
    pub(crate) fn ingest_incremental_update(
        &mut self,
        update: OrderBookUpdate,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if update.symbol != self.symbol {
            return Err("Wrong symbol update".into());
        }
        if update.timestamp <= self.timestamp {
            return Ok("Skipping update because it's old".into());
        }

        if update.previous_timestamp != self.timestamp {
            return Err("Missed an update in between".into());
        }

        self.timestamp = update.timestamp;

        Self::upsert_filter_and_sort(&mut self.asks, update.asks, self.max_levels, false);
        Self::upsert_filter_and_sort(&mut self.bids, update.bids, self.max_levels, true);

        Ok("Update successful".into())
    }

    // Prints the best 5 bids/asks of the orderbook.
    pub(crate) fn print_state(&self) {
        print!("\x1B[H\x1B[J");

        println!(
            "{:>4} | {:>14} | {:>14} | {:>14} | {:>14}",
            "#", "SIZE", "BID", "ASK", "SIZE"
        );
        println!("{}", "-".repeat(72));

        for index in 0..5 {
            let (bid_quantity, bid_price) = self
                .bids
                .get(index)
                .map(|bid| (bid.quantity.to_string(), bid.price.to_string()))
                .unwrap_or_else(|| ("-".to_string(), "-".to_string()));

            let (ask_price, ask_quantity) = self
                .asks
                .get(index)
                .map(|ask| (ask.price.to_string(), ask.quantity.to_string()))
                .unwrap_or_else(|| ("-".to_string(), "-".to_string()));

            println!(
                "{:>4} | {:>14} | {:>14} | {:>14} | {:>14}",
                index + 1,
                bid_quantity,
                bid_price,
                ask_price,
                ask_quantity
            );
        }
        io::stdout().flush().unwrap();
    }

    fn filter_and_sort(price_levels: &mut Vec<PriceLevel>, max_levels: usize, descending: bool) {
        price_levels.retain(|level| level.quantity != Decimal::ZERO);

        if descending {
            price_levels.sort_by(|left, right| right.price.cmp(&left.price));
        } else {
            price_levels.sort_by(|left, right| left.price.cmp(&right.price));
        }

        while price_levels.len() > max_levels {
            price_levels.pop();
        }
    }
    fn upsert_filter_and_sort(
        price_levels: &mut Vec<PriceLevel>,
        updates: Vec<PriceLevel>,
        max_levels: usize,
        descending: bool,
    ) {
        for update in updates {
            if let Some(index) = price_levels
                .iter()
                .position(|level| level.price == update.price)
            {
                price_levels[index].quantity = update.quantity;
            } else {
                price_levels.push(update);
            }
        }

        Self::filter_and_sort(price_levels, max_levels, descending);
    }
}
