use crate::server::network::{heartbeat::start_heartbeat_loop, packet_resolver::PacketResolver};
use rand::Rng;
use std::collections::HashMap;
use std::fs::write;
use std::iter::repeat_with;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use super::config::Config;
use super::game::dmf_map::DmfMap;
use super::game::player::Player;

pub struct Server {
    pub connected_players: Arc<Mutex<HashMap<i8, Player>>>,

    pub loaded_maps: Arc<Mutex<HashMap<String, DmfMap>>>,
    pub config: Arc<Config>,

    pub salt: String,
}

impl Server {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(Config::load("server-config.yml")?);
        let salt = generate_salt(16);
        let server = Server {
            connected_players: Arc::new(Mutex::new(HashMap::new())),
            loaded_maps: Arc::new(Mutex::new(HashMap::new())),
            config,
            salt,
        };

        return Ok(server);
    }

    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        let resolver = Arc::new(PacketResolver::new(Arc::clone(&self)));
        create_default_test_world().await?;

        tokio::spawn({
            let server = Arc::clone(&self);
            async move {
                start_heartbeat_loop(server).await;
            }
        });

        let resolver_clone = Arc::clone(&resolver);
        tokio::spawn(async move {
            resolver_clone.ping_players_loop().await;
        });

        let address = format!("{}:{}", self.config.addr, self.config.port);
        let listener = TcpListener::bind(&address).await?;
        println!("Starting server on {}", &address);

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
    let mut reader = Arc::new(Mutex::new(reader));
    let mut writer = Arc::new(Mutex::new(writer));
    let mut buf = [0; 1024];

    loop {
        let n = match reader.lock().await.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                println!("Error reading incoming data: {}", e);
                break;
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

        resolver.handle_packet(&buf[..n], Arc::clone(&writer)).await;
    }

    if let Err(e) = writer.lock().await.shutdown().await {
        println!("Erro closing writer: {}", e);
    }
    Ok(())
}

async fn create_default_test_world() -> io::Result<()> {
    let mut world = DmfMap::new(128, 4, 128, 256, 64, 256);

    for x in 0..256 {
        for z in 0..256 {
            for y in 0..31 {
                world.set_block(x, y, z, 0x01);
            }
            world.set_block(x, 31, z, 0x02);
        }
    }

    fs::create_dir_all("maps").await?;
    world.save_file("maps/default.dmf")
}
fn generate_salt(length: usize) -> String {
    const BASE62: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let mut rng = rand::thread_rng();

    repeat_with(|| BASE62[rng.gen_range(0..BASE62.len())] as char)
        .take(length)
        .collect()
}
