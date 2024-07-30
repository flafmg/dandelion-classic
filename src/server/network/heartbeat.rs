use std::{sync::Arc, time::Duration};

use reqwest::Client;
use tokio::time::sleep;

use crate::server::{config::Config, server::Server};

const HEARTBEAT_INTERVAL: u64 = 45;
const SERVER_SOFTWARE: &str = "&eDANDELION &70.0.1";

pub async fn start_heartbeat_loop(server: Arc<Server>) {
    loop {
        let server_clone = server.clone();
        if let Err(er) = send_heartbeats(server_clone).await {
            println!("ERROR: error sending heartbeats: {}", er);
        }
        sleep(Duration::from_secs(HEARTBEAT_INTERVAL)).await;
    }
}

async fn send_heartbeats(server: Arc<Server>) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let config = server.config.clone();
    let response = client
        .get(server.config.heartbeat_url.clone())
        .query(&[
            ("port", config.port.to_string()),
            ("max", config.max_players.to_string()),
            ("name", config.name.clone()),
            ("public", config.public.to_string()),
            ("version", "7".to_string()),
            ("salt", server.salt.clone()),
            ("users", server.connected_players.len().to_string()),
            ("software", SERVER_SOFTWARE.to_string()),
            ("web", "false".to_string()),
        ])
        .send()
        .await?;

    if response.status().is_success() {
        let response_body = response.text().await?;
    } else {
        println!("Heartbeat ERROR: {}", response.status());
    }

    Ok(())
}
