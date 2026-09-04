// use http::Uri;
use tungstenite::{
    connect,
    http::Uri,
    protocol::Message,
    ClientRequestBuilder,
};
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;

fn main() { 
    let uri: Uri = "wss://wss.woox.io/v3/public".parse().unwrap();
    let builder = ClientRequestBuilder::new(uri);
    let (mut socket, response) = connect(builder).unwrap();
    
    println!("Websocket creation response = {response:#?}");

    let timestamp = u64::try_from(SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_millis()).expect("Unix timestamp should fit in u64");

    let ping_cmd = format!(
    r#"{{"cmd":"PING","ts":{timestamp}}}"#
);  

    let started = std::time::Instant::now();
    socket.send(Message::text(ping_cmd)).unwrap();

    let msg = socket.read().unwrap();

    match msg {
        tungstenite::Message::Text(bytes) => {
            let response: Value = serde_json::from_str(bytes.as_str()).unwrap();

            let server_time = response["time"]
                .as_u64()
                .expect("PONG response should contain an integer time");

                println!("Response = {:#?}", &bytes);
                println!("Clock diff: {}ms", server_time - timestamp);
                println!("Measured Latency: {:?}", started.elapsed());
        }
        _ => {
            unreachable!();
        }
    }
}
