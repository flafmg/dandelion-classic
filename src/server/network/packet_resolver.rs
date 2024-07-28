use super::packet::PacketTrait;
use super::packet_stream::packet_reader::PacketReader;
use super::packet_stream::packet_writer::PacketWriter;
use super::packets::clientbound::{LevelFinalizePacket, LevelInitializePacket};
use super::packets::serverbound::PlayerIndentificationPacket;
use crate::server::game::dmf_map::DmfMap;
use crate::server::network::packets::clientbound::LevelDataChunkPacket;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::prelude::*;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

pub struct PacketResolver;

impl PacketResolver {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_packet(&mut self, data: &[u8], mut socket: TcpStream) {
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
                packet.resolve(&mut socket).await;

                self.send_level_data(&mut socket).await;
            }
            0x02 => {}
            _ => {
                println!("ERROR: Unknown packet ID: {}", packet_id);
            }
        }
    }

    async fn send_level_data(&self, socket: &mut TcpStream) {
        let mut world = DmfMap::load_file("maps/default.dmf").unwrap();
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
