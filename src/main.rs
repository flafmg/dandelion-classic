use server::server::start_server;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    start_server("0.0.0.0:25565").await
}
