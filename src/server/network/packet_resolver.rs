use super::packet::PacketTrait;
use super::packet_stream::packet_reader::PacketReader;
use super::packet_stream::packet_writer::PacketWriter;
use super::packets::clientbound::{
    LevelDataChunkPacket, LevelFinalizePacket, LevelInitializePacket,
};
use super::packets::serverbound::PlayerIndentificationPacket;
use crate::server::game::dmf_map::DmfMap;
use crate::server::game::player::Player;
use crate::server::network::packets::clientbound::PingPacket;
use crate::server::server::Server;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::sleep;

const PING_TIME: u64 = 5;

pub struct PacketResolver {
    pub server: Arc<Server>,
}

impl PacketResolver {
    pub fn new(server: Arc<Server>) -> Self {
        return Self { server };
    }
    pub async fn handle_packet(&self, data: &[u8], socket: Arc<Mutex<WriteHalf<TcpStream>>>) {
        if data.is_empty() {
            println!("ERROR: empty data received");
            return;
        }

        let packet_id = data[0];
        let mut reader = PacketReader::new(data);

        match packet_id {
            0x00 => {
                let mut packet = PlayerIndentificationPacket::new();
                packet.read(&mut reader);

                let player_id = self.get_last_id().await;
                let player = Player::new(
                    player_id,
                    packet.username.clone(),
                    "default".to_string(),
                    Arc::clone(&socket),
                );
                self.server
                    .connected_players
                    .lock()
                    .await
                    .insert(player_id, player.clone());

                let mut socket_guard = socket.lock().await;
                packet.resolve(&mut *socket_guard).await.unwrap();
                println!("Player {} connected!", player.get_name());

                self.send_level_data(&mut *socket_guard).await;
            }
            0x02 => {}
            _ => {
                println!("ERROR: Unknown packet ID: {}", packet_id);
            }
        }
    }

    pub async fn ping_players_loop(&self) {
        loop {
            let mut players = self.server.connected_players.lock().await;
            let mut remove_ids = Vec::new();
            for (id, player) in players.iter() {
                let mut socket = player.socket.lock().await;
                let mut ping_packet = PingPacket::new();
                let mut packet_writer = PacketWriter::new();
                ping_packet.write(&mut packet_writer);

                if let Err(er) = ping_packet.resolve(&mut socket).await {
                    println!("Player {} [{}] disconected", player.get_name(), id);
                    remove_ids.push(*id);
                }
            }
            for id in remove_ids {
                players.remove(&id);
            }

            sleep(Duration::from_secs(PING_TIME)).await;
        }
    }

    async fn get_last_id(&self) -> i8 {
        let players = self.server.connected_players.lock().await;
        for id in 0..=127 {
            let id = id as i8;
            if !players.contains_key(&id) {
                return id;
            }
        }
        return -1;
    }

    async fn send_level_data(&self, socket: &mut WriteHalf<TcpStream>) {
        let world = DmfMap::load_file("maps/default.dmf").unwrap();
        let block_data = world.blocks;

        let mut resolve_packet_response = LevelInitializePacket::new();
        let mut packet_writer = PacketWriter::new();
        resolve_packet_response.write(&mut packet_writer);
        resolve_packet_response.resolve(socket).await;

        let total_length = block_data.len();
        let mut prefixed_data = Vec::with_capacity(4 + total_length);
        prefixed_data.extend_from_slice(&(total_length as u32).to_be_bytes());
        prefixed_data.extend_from_slice(&block_data);

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&prefixed_data).unwrap();
        let compressed_data = encoder.finish().unwrap();

        let chunk_size = 1024;
        for (i, chunk) in compressed_data.chunks(chunk_size).enumerate() {
            println!("sending chunk {}", i);
            let chunk_length = chunk.len() as i16;
            let percent_complete = ((i * chunk_size + chunk_length as usize) as f32
                / compressed_data.len() as f32
                * 100.0) as u8;

            let mut chunk_data = chunk.to_vec();
            if chunk_data.len() < chunk_size {
                chunk_data.resize(chunk_size, 0x00);
            }

            let mut packet = LevelDataChunkPacket::new(chunk_length, chunk_data, percent_complete);
            let mut packet_writer = PacketWriter::new();
            packet.write(&mut packet_writer);

            if let Err(e) = packet.resolve(socket).await {
                println!("Error sending chunk {}: {}", i, e);
                break;
            }
        }

        let mut level_finalize = LevelFinalizePacket::new(world.x_size, world.y_size, world.z_size);
        let mut packet_writer = PacketWriter::new();
        level_finalize.write(&mut packet_writer);
        level_finalize.resolve(socket).await;
    }
}
