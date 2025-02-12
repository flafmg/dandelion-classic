use std::sync::Arc;

use server::server::Server;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Arc::new(Server::new().await?);
    server.start().await?;

    Ok(())
}
