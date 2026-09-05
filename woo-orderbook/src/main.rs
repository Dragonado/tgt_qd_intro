use std::time::Instant;

mod listener;
mod order_book;

use std::io::{self, Write};

use crate::listener::WooListener;
use crate::order_book::OrderBook;

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Self {
        // Enter alternate screen.
        print!("\x1B[?1049h");
        io::stdout().flush().unwrap();
        Self
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Show cursor and restore the normal terminal screen.
        print!("\x1B[?1049l");
        let _ = io::stdout().flush();
    }
}

// impl OrderBook {
//     fn snapshot_url(&self) -> Result<reqwest::Url, Box<dyn std::error::Error>> {
//         let mut url = reqwest::Url::parse("https://api.woox.io/v3/public/orderbook")?;

//         url.query_pairs_mut()
//             .append_pair("symbol", &self.symbol)
//             .append_pair("maxLevel", &self.max_levels.to_string())
//             .append_pair("rpi", "true");

//         Ok(url)
//     }

//     fn init_from_snapshot(&mut self, snapshot: Value) -> Result<(), Box<dyn std::error::Error>> {
//         let url = self.snapshot_url()?;
//         let response = reqwest::blocking::get(url)?.error_for_status()?.text()?;
//         let parsed_body: Value = serde_json::from_str(response.as_str())?;

//         if parsed_body["success"].as_bool() != Some(true) {
//             return Err("request was not successful".into());
//         }

//         self.timestamp = parsed_body["timestamp"].as_u64().ok_or_else(|| {
//             Error::new(
//                 ErrorKind::InvalidData,
//                 "snapshot timestamp is missing or is not a u64",
//             )
//         })?;

//         self.asks = Self::parse_order("asks", &parsed_body["data"])?;
//         self.bids = Self::parse_order("bids", &parsed_body["data"])?;

//         self.asks.sort();
//         self.bids.sort();

//         Ok(())
//     }
// }

fn main() {
    let _terminal_guard = TerminalGuard::enter();

    let mut order_book = OrderBook::new("PERP_ETH_USDT", 5);

    let mut listener = WooListener::connect().unwrap();

    listener
        .subscribe("orderbookupdaterpi@PERP_ETH_USDT@50")
        .unwrap();

    order_book.print_state();

    let mut started = std::time::Instant::now();
    loop {
        started = Instant::now();
        let update = listener.wait_and_get_update().unwrap();
        order_book.ingest_incremental_update(update).unwrap();
        order_book.print_state();
        println!("Incremental update RTT: {:?}", started.elapsed());
    }
}
