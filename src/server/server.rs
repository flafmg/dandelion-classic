use crate::server::network::{heartbeat::start_heartbeat_loop, packet_resolver::PacketResolver};
use dashmap::DashMap;
use rand::Rng;
use std::iter::repeat_with;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

use super::config::Config;
use super::game::dmf_map::DmfMap;
use super::game::player::Player;
use super::map_builder::{Dimensions, MapBuilder, NoiseLayer, PresetParams};
use super::maps;

pub struct Server {
    pub connected_players: Arc<DashMap<i8, Player>>,
    pub loaded_maps: Arc<DashMap<String, DmfMap>>,
    pub config: Arc<Config>,
    pub salt: String,
}

impl Server {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(Config::load("server-config.yml")?);
        let salt = generate_salt(16);
        let server = Server {
            connected_players: Arc::new(DashMap::new()),
            loaded_maps: Arc::new(DashMap::new()),
            config,
            salt,
        };

        Ok(server)
    }

    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        println!("Initializing server...");

        maps::load_all_maps_in("maps", Arc::clone(&self.loaded_maps)).await?;
        tokio::spawn({
            let maps = Arc::clone(&self.loaded_maps);
            async move {
                maps::save_all_loop(maps).await;
            }
        });

        tokio::spawn({
            let server = Arc::clone(&self);
            async move {
                start_heartbeat_loop(server).await;
            }
        });

        let resolver = Arc::new(PacketResolver::new(Arc::clone(&self)));
        tokio::spawn({
            let resolver_clone = Arc::clone(&resolver);
            async move {
                resolver_clone.ping_players_loop().await;
            }
        });
        tokio::spawn({
            let resolver_clone = Arc::clone(&resolver);
            async move {
                resolver_clone.send_to_all_queued().await;
            }
        });

        let address = format!("{}:{}", self.config.addr, self.config.port);
        let listener = TcpListener::bind(&address).await?;
        println!("Server started on {}", &address);

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("Client connected: {}", addr);

            let resolver_clone = Arc::clone(&resolver);
            tokio::spawn(handle_client(socket, resolver_clone));
        }
    }
}

async fn handle_client(
    socket: TcpStream,
    resolver: Arc<PacketResolver>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut reader, mut writer) = tokio::io::split(socket);
    let reader = Arc::new(RwLock::new(reader));
    let writer = Arc::new(RwLock::new(writer));
    let mut buf = [0; 1024];

    loop {
        let n = match reader.write().await.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading incoming data: {}", e);
                break;
            }
        };

        resolver.handle_packet(&buf[..n], Arc::clone(&writer)).await;
    }

    if let Err(e) = writer.write().await.shutdown().await {
        eprintln!("Error closing writer: {}", e);
    }

    Ok(())
}

fn generate_salt(length: usize) -> String {
    const BASE62: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let mut rng = rand::thread_rng();

    repeat_with(|| BASE62[rng.gen_range(0..BASE62.len())] as char)
        .take(length)
        .collect()
}
