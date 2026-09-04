use std::time::Instant;

use serde_json::Value;
use tungstenite::{ClientRequestBuilder, connect, http::Uri, protocol::Message};

#[derive(Debug)]
struct Order {
    price: String,
    quantity: String,
}

#[derive(Debug)]
struct OrderBook {
    timestamp: u64,
    symbol: String,
    max_order_size: u64,
    orders: Vec<Order>,
}

impl OrderBook {
    fn new(symbol: &str, max_order_size: u64) -> Self {
        Self {
            timestamp: 0,
            symbol: symbol.to_string(),
            max_order_size,
            orders: Vec::new(),
        }
    }

    fn init_snapshot(&self, snapshot_cmd: &str) -> Result<(), String> {
        println!("{snapshot_cmd}");
        Ok(())
    }
}

fn main() {
    let order_book = OrderBook::new("PERP_ETH_USDT", 5);
    let snapshot_cmd =
        "https://api.woo.org/v3/public/orderbook?symbol=PERP_ETH_USDT&maxLevel=5&rpi=true";
    order_book.init_snapshot(snapshot_cmd).unwrap();
    println!("{order_book:#?}");
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
