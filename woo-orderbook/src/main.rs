use std::{sync::mpsc::sync_channel, time::Instant};

mod listener;

mod order_book;

use crate::order_book::{Info, OrderBook, OrderBookUpdate};
use std::io::{self, Write};
use std::thread;

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
    println!("Order book structure:");
    order_book.info();

    println!("\nStart ingesting PERP_ETH_USDT data? (y/n): ");
    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    if input.trim() != "y" {
        return;
    }

    // Get inital snapshot.
    order_book = WooListener::wait_and_get_snapshot("PERP_ETH_USDT", 50).unwrap();

    // Using synchronous channel because we only have 1 producer in the MPSC queue.
    let (tx, rx) = sync_channel::<OrderBookUpdate>(1024);

    thread::spawn(move || {
        let mut listener = WooListener::connect().unwrap();
        listener
            .subscribe("orderbookupdaterpi@PERP_ETH_USDT@50")
            .unwrap();
        loop {
            tx.send(listener.wait_and_get_update().unwrap()).unwrap();
        }
    });

    let mut started;
    loop {
        started = Instant::now();
        let received = rx.recv();
        let status;
        match received {
            Ok(update) => match order_book.ingest_incremental_update(update) {
                Err(err) => {
                    order_book = WooListener::wait_and_get_snapshot("PERP_ETH_USDT", 50).unwrap();
                    status = format!("Missed an update in between, fetching from snapshot: {err}");
                }
                Ok(msg) => {
                    status = format!("Recieved update from subscription: {msg}");
                }
            },
            Err(err) => {
                println!("Subscription connection failed! Try again. Err = {err}");
                return;
            }
        }

        order_book.print_state();
        println!();
        println!("Status: {status}");
        println!("Update wait time: {:?}", started.elapsed());
    }
}
