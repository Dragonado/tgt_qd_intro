// use http::Uri;
use tungstenite::{http::Uri, connect, ClientRequestBuilder};

fn main() { 
    let uri: Uri = "wss://wss.woox.io/v3/public".parse().unwrap();
    let builder = ClientRequestBuilder::new(uri);
    let socket = connect(builder).unwrap();

    println!("socket = {socket:?}");
}
