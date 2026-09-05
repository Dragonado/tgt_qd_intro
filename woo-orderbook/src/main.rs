use std::time::Instant;

use std::io::{self, Error, ErrorKind, Write};

use serde_json::Value;
use tungstenite::{ClientRequestBuilder, connect, http::Uri, protocol::Message};

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

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct PriceLevel {
    price: String,
    quantity: String,
}

#[derive(Debug, Default)]
struct OrderBook {
    timestamp: u64,
    symbol: String,
    max_levels: usize,
    asks: Vec<PriceLevel>,
    bids: Vec<PriceLevel>,
}

impl OrderBook {
    fn new(symbol: &str, max_levels: usize) -> Self {
        Self {
            timestamp: 0,
            symbol: symbol.to_string(),
            max_levels,
            asks: Vec::new(),
            bids: Vec::new(),
        }
    }

    fn print_state(&self) {
        print!("\x1B[H\x1B[J");

        println!(
            "{:>14} | {:>14} | {:>14} | {:>14}",
            "SIZE", "BID", "ASK", "SIZE"
        );
        println!("{}", "-".repeat(65));

        for (bid, ask) in self
            .bids
            .iter()
            .rev()
            .take(self.max_levels)
            .zip(self.asks.iter().take(self.max_levels))
        {
            println!(
                "{:>14} | {:>14} | {:>14} | {:>14}",
                bid.quantity, bid.price, ask.price, ask.quantity
            );
        }
        io::stdout().flush().unwrap();
    }

    fn snapshot_url(&self) -> Result<reqwest::Url, Box<dyn std::error::Error>> {
        let mut url = reqwest::Url::parse("https://api.woox.io/v3/public/orderbook")?;

        url.query_pairs_mut()
            .append_pair("symbol", &self.symbol)
            .append_pair("maxLevel", &self.max_levels.to_string())
            .append_pair("rpi", "true");

        Ok(url)
    }

    fn parse_order(
        side_key: &str,
        data: &Value,
    ) -> Result<Vec<PriceLevel>, Box<dyn std::error::Error>> {
        let mut price_levels = Vec::new();
        for parsed_price_level in data[side_key].as_array().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("{side_key} is missing or is not an array"),
            )
        })? {
            let price_level = PriceLevel {
                price: String::from(parsed_price_level["price"].as_str().ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("{side_key} is missing price or is not a string"),
                    )
                })?),

                quantity: String::from(parsed_price_level["quantity"].as_str().ok_or_else(
                    || {
                        Error::new(
                            ErrorKind::InvalidData,
                            format!("{side_key} is missing quantity or is not a string"),
                        )
                    },
                )?),
            };

            price_levels.push(price_level);
        }
        Ok(price_levels)
    }

    fn init_from_snapshot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.snapshot_url()?;
        let response = reqwest::blocking::get(url)?.error_for_status()?.text()?;
        let parsed_body: Value = serde_json::from_str(response.as_str())?;

        if parsed_body["success"].as_bool() != Some(true) {
            return Err("request was not successful".into());
        }

        self.timestamp = parsed_body["timestamp"].as_u64().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "snapshot timestamp is missing or is not a u64",
            )
        })?;

        self.asks = Self::parse_order("asks", &parsed_body["data"])?;
        self.bids = Self::parse_order("bids", &parsed_body["data"])?;

        self.asks.sort();
        self.bids.sort();

        Ok(())
    }
}

fn main() {
    let mut order_book = OrderBook::new("PERP_ETH_USDT", 5);
    order_book.init_from_snapshot().unwrap();

    let uri: Uri = "wss://wss.woox.io/v3/public".parse().unwrap();
    let builder = ClientRequestBuilder::new(uri);
    let mut started = std::time::Instant::now();
    let (mut socket, response) = connect(builder).unwrap();

    println!("Websocket creation response = {response:#?}");
    println!("Websocket connection RTT: {:?}\n", started.elapsed());

    let subscribe_cmd = serde_json::json!({
        "cmd": "SUBSCRIBE",
        "params": ["orderbookupdaterpi@PERP_ETH_USDT@50"]
    });

    started = Instant::now();
    socket
        .send(Message::text(subscribe_cmd.to_string()))
        .unwrap();

    let subscription_ack_message = socket.read().unwrap();

    match subscription_ack_message {
        tungstenite::Message::Text(bytes) => {
            let response: Value = serde_json::from_str(bytes.as_str()).unwrap();

            println!("{}", serde_json::to_string_pretty(&response).unwrap());
            println!("Subscription connection RTT: {:?}", started.elapsed());
        }
        _ => {
            unreachable!();
        }
    }

    let _terminal = TerminalGuard::enter();
    order_book.print_state();

    loop {
        started = Instant::now();
        let incremental_update = socket.read().unwrap();
        match incremental_update {
            tungstenite::Message::Text(bytes) => {
                let response: Value = serde_json::from_str(bytes.as_str()).unwrap();

                // println!("{}", serde_json::to_string_pretty(&response).unwrap());
                order_book.print_state();
                println!("Incremental update RTT: {:?}", started.elapsed());
            }
            _ => {
                unreachable!();
            }
        }
    }
}
