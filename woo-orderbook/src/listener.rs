use crate::order_book::{OrderBookUpdate, PriceLevel};
use serde_json::{Map, Value};
use std::io::{Error, ErrorKind};
use std::net::TcpStream;
use tungstenite::{ClientRequestBuilder, WebSocket, connect, http::Uri, protocol::Message, stream};

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

    fn parse_price_levels(
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
            let parsed_price_level_arr = parsed_price_level.as_array().ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("{side_key} is not an array of [price, quantity]"),
                )
            })?;

            let price_level = PriceLevel {
                price: String::from(parsed_price_level[0].as_str().ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("{side_key} is missing price"),
                    )
                })?),
                quantity: String::from(parsed_price_level_arr[1].as_str().ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidData,
                        format!("{side_key} is missing quantity"),
                    )
                })?),
            };

            price_levels.push(price_level);
        }
        Ok(price_levels)
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
                    asks: Self::parse_price_levels("asks", data_json)?,
                    bids: Self::parse_price_levels("bids", data_json)?,
                })
            }
            _ => Err("Update unsuccessful.".into()),
        }
    }
}
