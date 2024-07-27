use crate::server::network::{heartbeat::start_heartbeat_loop, packet_resolver::PacketResolver};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{io::AsyncReadExt, net::TcpListener, net::TcpStream};

pub async fn start_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let packet_resolver = Arc::new(Mutex::new(PacketResolver::new()));

    tokio::spawn(async move {
        start_heartbeat_loop().await;
    });

    let listener = TcpListener::bind(addr).await?;
    println!("Starting server on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);

        let resolver_clone = Arc::clone(&packet_resolver);
        tokio::spawn(handle_client(socket, resolver_clone));
    }
}

async fn handle_client(
    mut socket: TcpStream,
    resolver: Arc<Mutex<PacketResolver>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = [0; 1024];

    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return Ok(());
            }
            Ok(n) => n,
            Err(e) => {
                println!("Error reading incoming data: {}", e);
                return Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>);
            }
        };

        println!("Incoming data: ");

        let hex = buf[..n]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join(" ");
        println!(" HEX: {}", hex);

        match String::from_utf8(buf[..n].to_vec()) {
            Ok(s) => println!(" STR: {}", s),
            Err(_) => println!(" STR: ?"),
        }

        let mut resolver = resolver.lock().await;
        resolver.handle_packet(&buf[..n], socket).await;

        return Ok(());
    }
}
