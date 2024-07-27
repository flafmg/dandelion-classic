//TODO: add the heartbeat server to a config file (maybe a server-config.yml ?)

//doing this in the messy way only to see if it will work

use std::time::Duration;

use reqwest::Client;
use tokio::time::sleep;

const HEARTBEAT_INTERVAL: u64 = 45;
const HEARTBEAT_URL: &str = "https://www.classicube.net/server/heartbeat";
const SALT: &str = "wo6kVAHjxoJcInKx";

//put this in a struct + impl later, oop in rust babe B)
pub async fn start_heartbeat_loop() {
    println!("starting hearbeat loop");
    loop {
        if let Err(er) = send_heartbeats().await {
            println!("ERROR: error sending heartbeats: {}", er);
        }
        sleep(Duration::from_secs(HEARTBEAT_INTERVAL)).await;
    }
}

async fn send_heartbeats() -> Result<(), reqwest::Error> {
    let client = Client::new();

    let response = client
        .get(HEARTBEAT_URL)
        .query(&[
            ("port", "25565"),
            ("max", "32"),
            ("name", "dandelion test server"),
            ("public", "True"),
            ("version", "7"),
            ("salt", SALT),
            ("users", "0"),
            ("software", "dandelion"),
            ("web", "True"),
        ])
        .send()
        .await?;

    if response.status().is_success() {
        let response_body = response.text().await?;
        println!("Heartbeat respose: {}", response_body);
    } else {
        println!("Hearbeat ERROR: {}", response.status());
    }

    return Ok(());
}
