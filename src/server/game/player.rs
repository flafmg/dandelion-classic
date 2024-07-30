use crate::server::game::dmf_map::DmfMap;
use crate::server::network::packets::clientbound::{
    SendMessagePacket, SetPositionAndOrientationPacket,
};
use crate::server::network::{
    packet::PacketTrait,
    packet_stream::packet_writer::PacketWriter,
    packets::clientbound::{LevelDataChunkPacket, LevelFinalizePacket, LevelInitializePacket},
};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;
use std::sync::Arc;
use tokio::{io::WriteHalf, net::TcpStream, sync::RwLock};

#[derive(Clone, Debug)]
pub struct Player {
    id: i8,
    name: String,
    pub current_map: String,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub yaw: u8,
    pub pitch: u8,
    pub socket: Arc<RwLock<WriteHalf<TcpStream>>>,
}

impl Player {
    pub fn new(
        id: i8,
        name: String,
        current_map: String,
        socket: Arc<RwLock<WriteHalf<TcpStream>>>,
    ) -> Self {
        Self {
            id,
            name,
            current_map,
            x: 0,
            y: 0,
            z: 0,
            yaw: 0,
            pitch: 0,
            socket,
        }
    }

    pub fn get_id(&self) -> i8 {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_current_world(&self) -> &str {
        &self.current_map
    }

    pub fn set_pos(&mut self, x: i16, y: i16, z: i16, pitch: u8, yaw: u8) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.pitch = pitch;
        self.yaw = yaw;
    }
    pub async fn teleport(&mut self, x: i16, y: i16, z: i16, pitch: u8, yaw: u8) {
        self.set_pos(x, y, z, pitch, yaw);
        let mut set_position = SetPositionAndOrientationPacket::new(-1, x, y, z, pitch, yaw);
        set_position.write(&mut PacketWriter::new());
        let mut socket = self.socket.write().await;
        set_position.resolve(&mut socket).await;
    }
    pub async fn send_message(&self, msg: &str) {
        let mut message_packet = SendMessagePacket::new(-1, msg.to_string());
        message_packet.write(&mut PacketWriter::new());
        let mut socket = self.socket.write().await;
        message_packet.resolve(&mut socket).await;
    }
    pub async fn send_to_level(&self, map: &DmfMap) {
        let block_data = &map.blocks;

        let mut socket_guard = self.socket.write().await;

        let mut resolve_packet_response = LevelInitializePacket::new();
        let mut packet_writer = PacketWriter::new();
        resolve_packet_response.write(&mut packet_writer);
        resolve_packet_response
            .resolve(&mut *socket_guard)
            .await
            .unwrap();

        let total_length = block_data.len();
        let mut prefixed_data = Vec::with_capacity(4 + total_length);
        prefixed_data.extend_from_slice(&(total_length as u32).to_be_bytes());
        prefixed_data.extend_from_slice(&block_data);

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&prefixed_data).unwrap();
        let compressed_data = encoder.finish().unwrap();

        let chunk_size = 1024;
        for (i, chunk) in compressed_data.chunks(chunk_size).enumerate() {
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

            if let Err(e) = packet.resolve(&mut *socket_guard).await {
                eprintln!("Error sending chunk {}: {}", i, e);
                break;
            }
        }

        let mut level_finalize = LevelFinalizePacket::new(map.x_size, map.y_size, map.z_size);
        let mut packet_writer = PacketWriter::new();
        level_finalize.write(&mut packet_writer);
        level_finalize.resolve(&mut *socket_guard).await.unwrap();
    }
}
