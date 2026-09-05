use std::time::Instant;

mod listener;
mod order_book;

use crate::order_book::{Info, OrderBook};
use std::io::{self, Write};

use crate::listener::WooListener;

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

fn main() {
    let _terminal_guard = TerminalGuard::enter();

    let mut order_book = OrderBook::new("PERP_ETH_USDT".to_string(), 50);

    let mut listener = WooListener::connect().unwrap();

    listener
        .subscribe("orderbookupdaterpi@PERP_ETH_USDT@50")
        .unwrap();

    order_book.info();

    let mut started;
    loop {
        started = Instant::now();
        let update = listener.wait_and_get_update().unwrap();
        let status;
        match order_book.ingest_incremental_update(update) {
            Err(error) => {
                order_book = WooListener::wait_and_get_snapshot("PERP_ETH_USDT", 50).unwrap();
                status = format!("FRESH SNAPSHOT: {error}");
            }
            Ok(()) => {
                status = String::from("UPDATED");
            }
        }
        order_book.print_state();
        println!();
        println!("Status: {status}");
        println!("Incremental update RTT: {:?}", started.elapsed());
    }
}
