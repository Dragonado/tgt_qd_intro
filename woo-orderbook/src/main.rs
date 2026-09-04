use std::time::Instant;

use std::io::{Error, ErrorKind};

use serde_json::{Number, Value};
use tungstenite::{ClientRequestBuilder, connect, http::Uri, protocol::Message};

#[derive(Debug, Default)]
struct Order {
    price: String,
    quantity: String,
}

#[derive(Debug, Default)]
struct OrderBook {
    timestamp: u64,
    symbol: String,
    max_order_size: u64,
    asks: Vec<Order>,
    bids: Vec<Order>,
}

impl OrderBook {
    fn new(symbol: &str, max_order_size: u64) -> Self {
        Self {
            timestamp: 0,
            symbol: symbol.to_string(),
            max_order_size,
            asks: Vec::new(),
            bids: Vec::new(),
        }
    }

    fn init_snapshot(&mut self, snapshot_cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        // optionally assert snapshot is asking for same symbol and max_size.
        let body = reqwest::blocking::get(snapshot_cmd)?.text()?;
        let parsed_body: Value = serde_json::from_str(body.as_str())?;

        if parsed_body["success"].as_bool() != Some(true) {
            return Err("request was not successful".into());
        }

        self.timestamp = parsed_body["timestamp"].as_u64().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                "snapshot timestamp is missing or is not a u64",
            )
        })?;

        self.asks = Vec::new();
        self.bids = Vec::new();

        for parsed_ask in parsed_body["data"]["asks"].as_array().ok_or_else(|| {
            Error::new(ErrorKind::InvalidData, "asks is missing or is not an array")
        })? {
            let order = Order {
                price: String::from(parsed_ask["price"].as_str().ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        "ask is missing price or is not a string",
                    )
                })?),

                quantity: String::from(parsed_ask["quantity"].as_str().ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        "ask is missing quantity or is not a string",
                    )
                })?),
            };

            self.asks.push(order);
        }

        for parsed_bid in parsed_body["data"]["bids"].as_array().ok_or_else(|| {
            Error::new(ErrorKind::InvalidData, "bids is missing or is not an array")
        })? {
            let order = Order {
                price: String::from(parsed_bid["price"].as_str().ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        "bid is missing price or is not a string",
                    )
                })?),

                quantity: String::from(parsed_bid["quantity"].as_str().ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        "bid is missing quantity or is not a string",
                    )
                })?),
            };

            self.bids.push(order);
        }

        Ok(())
    }
}

fn main() {
    let mut order_book = OrderBook::new("PERP_ETH_USDT", 5);
    let snapshot_cmd =
        "https://api.woo.org/v3/public/orderbook?symbol=PERP_ETH_USDT&maxLevel=5&rpi=true";
    order_book.init_snapshot(snapshot_cmd).unwrap();

    println!("order_book = {order_book:?}");

    // let uri: Uri = "wss://wss.woox.io/v3/public".parse().unwrap();
    // let builder = ClientRequestBuilder::new(uri);
    // let mut started = std::time::Instant::now();
    // let (mut socket, response) = connect(builder).unwrap();

    // println!("Websocket creation response = {response:#?}");
    // println!("Websocket connection RTT: {:?}\n", started.elapsed());

    // let subscribe_cmd = serde_json::json!({
    //     "cmd": "SUBSCRIBE",
    //     "params": ["orderbookupdaterpi@PERP_ETH_USDT@5"]
    // });

    // started = Instant::now();
    // socket
    //     .send(Message::text(subscribe_cmd.to_string()))
    //     .unwrap();

    // let msg = socket.read().unwrap();

    // match msg {
    //     tungstenite::Message::Text(bytes) => {
    //         let response: Value = serde_json::from_str(bytes.as_str()).unwrap();

    //         println!("{}", serde_json::to_string_pretty(&response).unwrap());
    //         println!("Subscription connection RTT: {:?}", started.elapsed());
    //     }
    //     _ => {
    //         unreachable!();
    //     }
    // }
}
