use crate::order_book::{OrderBook, OrderBookUpdate, PriceLevel};
use serde_json::{Map, Value};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use tungstenite::{ClientRequestBuilder, WebSocket, connect, http::Uri, protocol::Message, stream};

use rust_decimal::Decimal;

pub(crate) struct WooListener {
    socket: WebSocket<stream::MaybeTlsStream<TcpStream>>,
}

impl WooListener {
    pub(crate) fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let uri: Uri = "wss://wss.woox.io/v3/public".parse().unwrap();
        let builder = ClientRequestBuilder::new(uri);
        let (socket, _) = connect(builder)?;

        Ok(WooListener { socket })
    }

    pub(crate) fn subscribe(&mut self, topic: &str) -> Result<(), Box<dyn std::error::Error>> {
        let command = serde_json::json!({
            "cmd": "SUBSCRIBE",
            "params": [topic],
        });
        self.socket.send(Message::text(command.to_string()))?;

        let subscription_ack_message = self.socket.read()?;

        match subscription_ack_message {
            tungstenite::Message::Text(bytes) => {
                let response: Value = serde_json::from_str(bytes.as_str())?;

                if response["success"].as_bool() != Some(true) {
                    return Err("Subscription unsuccessful.".into());
                }

                println!("{}", serde_json::to_string_pretty(&response)?);
                Ok(())
            }
            _ => Err("Subscription unsuccessful.".into()),
        }
    }

    fn parse_price_levels_from_update(
        side_key: &str,
        data: &Map<String, Value>,
    ) -> Result<Vec<PriceLevel>, Box<dyn std::error::Error>> {
        let mut price_levels = Vec::new();
        for parsed_price_level in data[side_key].as_array().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("{side_key} is missing or is not an array"),
            )
        })? {
            let price_level = PriceLevel {
                price: parsed_price_level
                    .get(0)
                    .and_then(Value::as_str)
                    .ok_or("price is missing or is not a string")?
                    .parse::<Decimal>()?,

                quantity: parsed_price_level
                    .get(1)
                    .and_then(Value::as_str)
                    .ok_or("quantity is missing or is not a string")?
                    .parse::<Decimal>()?,
            };

            price_levels.push(price_level);
        }
        Ok(price_levels)
    }

    fn parse_price_levels_from_snapshot(
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
                price: parsed_price_level["price"]
                    .as_str()
                    .ok_or("price is missing or is not a string")?
                    .parse::<Decimal>()?,
                quantity: parsed_price_level["quantity"]
                    .as_str()
                    .ok_or("quantity is missing or is not a string")?
                    .parse::<Decimal>()?,
            };

            price_levels.push(price_level);
        }
        Ok(price_levels)
    }

    // {"success":true,"timestamp":1788590426550,"data":{"asks":[{"price":"2457.1","quantity":"12.4957"},{"price":"2458.7","quantity":"42.783"},{"price":"2467.7","quantity":"37.077"},{"price":"2478.6","quantity":"37.081"},{"price":"2492.6","quantity":"42.81"}],"bids":[{"price":"2441.6","quantity":"11.413"},{"price":"2441.5","quantity":"1.051"},{"price":"2440.9","quantity":"42.796"},{"price":"2432.7","quantity":"37.083"},{"price":"2421.8","quantity":"37.079"}]}}
    pub(crate) fn wait_and_get_snapshot(
        symbol: &str,
        max_levels: usize,
    ) -> Result<OrderBook, Box<dyn std::error::Error>> {
        let mut url = reqwest::Url::parse("https://api.woox.io/v3/public/orderbook")?;

        url.query_pairs_mut()
            .append_pair("symbol", symbol)
            .append_pair("maxLevel", "50")
            .append_pair("rpi", "true");

        let response = reqwest::blocking::get(url)?.error_for_status()?.text()?;
        let parsed_body: Value = serde_json::from_str(response.as_str())?;

        if parsed_body["success"].as_bool() != Some(true) {
            return Err("request was not successful".into());
        }

        Ok(OrderBook::from_snapshot(
            symbol.to_string(),
            max_levels,
            parsed_body["timestamp"].as_u64().ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    "snapshot timestamp is missing or is not a u64",
                )
            })?,
            Self::parse_price_levels_from_snapshot("asks", &parsed_body["data"])?,
            Self::parse_price_levels_from_snapshot("bids", &parsed_body["data"])?,
        ))
    }
    //   "data": {
    //     "asks": [],
    //     "bids": [
    //       [
    //         "2450.7",
    //         "25.9971"
    //       ]
    //     ],
    //     "prevTs": 1788575462800,
    //     "s": "PERP_ETH_USDT",
    //     "ts": 1788575462850
    //   },
    //   "topic": "orderbookupdaterpi@PERP_ETH_USDT@50",
    //   "ts": 1788575462851
    // }
    pub(crate) fn wait_and_get_update(
        &mut self,
    ) -> Result<OrderBookUpdate, Box<dyn std::error::Error>> {
        let response_msg = self.socket.read()?;
        match response_msg {
            tungstenite::Message::Text(bytes) => {
                let response_json: Value = serde_json::from_str(bytes.as_str())?;

                let data_json = response_json["data"]
                    .as_object()
                    .ok_or("update data is missing or is not an object")?;

                Ok(OrderBookUpdate {
                    symbol: data_json["s"]
                        .as_str()
                        .ok_or_else(|| {
                            Error::new(ErrorKind::InvalidData, "update is missing string s")
                        })?
                        .to_string(),
                    previous_timestamp: data_json["prevTs"].as_u64().ok_or_else(|| {
                        Error::new(ErrorKind::InvalidData, "update is missing integer prevTs")
                    })?,
                    timestamp: data_json["ts"].as_u64().ok_or_else(|| {
                        Error::new(ErrorKind::InvalidData, "update is missing integer ts")
                    })?,
                    asks: Self::parse_price_levels_from_update("asks", data_json)?,
                    bids: Self::parse_price_levels_from_update("bids", data_json)?,
                })
            }
            _ => Err("Update unsuccessful.".into()),
        }
    }
}
