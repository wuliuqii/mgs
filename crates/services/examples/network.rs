use std::process::exit;

use futures_signals::signal::SignalExt;
use futures_util::StreamExt;
use services::network;

#[tokio::main]
async fn main() {
    let client = network::Subscriber::new().await.unwrap();
    // Create a clone of client to be moved into the spawned task
    let client_clone = client.clone();
    tokio::spawn(async move {
        client_clone.run().await.unwrap();
        exit(-1);
    });
    let mut signal = client.subscribe().to_stream();
    while let Some(state) = signal.next().await {
        println!("State: {:?}", state);
    }
}
