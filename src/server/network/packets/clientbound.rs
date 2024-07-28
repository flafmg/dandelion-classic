use std::io::Write;
use std::time::Duration;

use async_trait::async_trait;
use flate2::write::GzEncoder;
use flate2::Compression;
use tokio::net::TcpStream;
use tokio::{io::AsyncWriteExt, time::sleep};

use crate::server::{
    network::{
        packet::PacketTrait,
        packet_stream::{packet_reader::PacketReader, packet_writer::PacketWriter},
    },
    world::dmf_world::DmfWorld,
};

// server identification packet

pub struct ServerIdentificationPacket {
    data: Vec<u8>,
    protocol_version: u8,
    server_name: String,
    server_motd: String,
    user_type: u8,
}
impl ServerIdentificationPacket {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            protocol_version: 0x07,
            server_name: "dandelion".to_string(),
            server_motd: "hello mom!".to_string(),
            user_type: 0x64,
        }
    }
}

#[async_trait]
impl PacketTrait for ServerIdentificationPacket {
    fn packet_id(&self) -> u8 {
        0x00
    }
    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_byte(self.protocol_version);
        writer.write_string(self.server_name.as_str());
        writer.write_string(self.server_motd.as_str());
        writer.write_byte(self.user_type);
        self.data = writer.to_bytes().clone();
    }
    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        let mut resolve_packet_response = LevelInitializePacket::new();
        let mut packet_writer = PacketWriter::new();
        resolve_packet_response.write(&mut packet_writer);
        resolve_packet_response.resolve(socket).await?;

        Ok(())
    }
}

// level initialize packet

pub struct LevelInitializePacket {
    data: Vec<u8>,
}
impl LevelInitializePacket {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

#[async_trait]
impl PacketTrait for LevelInitializePacket {
    fn packet_id(&self) -> u8 {
        0x02
    }
    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        self.data = writer.to_bytes().clone();
    }
    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;

        let mut world = DmfWorld::load_file("maps/default.dmf").unwrap();
        let block_data = world.blocks;

        let total_length = block_data.len();
        let mut prefixed_data = Vec::with_capacity(4 + total_length);
        prefixed_data.extend_from_slice(&(total_length as u32).to_be_bytes());
        prefixed_data.extend_from_slice(&block_data);

        //im dumb i needed to compress this shit
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

            if let Err(e) = packet.resolve(socket).await {
                println!("Error sending chunk {}: {}", i, e);
                break;
            }

            sleep(Duration::from_secs(1)).await;
        }

        let mut level_finalize = LevelFinalizePacket::new(world.x_size, world.y_size, world.z_size);
        let mut packet_writer = PacketWriter::new();
        level_finalize.write(&mut packet_writer);
        level_finalize.resolve(socket).await?;

        Ok(())
    }
}

// level data chunk packet

pub struct LevelDataChunkPacket {
    data: Vec<u8>,
    chunk_length: i16,
    chunk_data: Vec<u8>,
    completed: u8,
}

impl LevelDataChunkPacket {
    pub fn new(chunk_length: i16, chunk_data: Vec<u8>, completed: u8) -> Self {
        Self {
            data: Vec::new(),
            chunk_length,
            chunk_data,
            completed,
        }
    }
}

#[async_trait]
impl PacketTrait for LevelDataChunkPacket {
    fn packet_id(&self) -> u8 {
        0x03
    }
    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_short(self.chunk_length);
        writer.write_byte_array(self.chunk_data.as_slice(), 1024);
        writer.write_byte(self.completed);
        self.data = writer.to_bytes().clone();
    }
    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

// level finalize packet

pub struct LevelFinalizePacket {
    data: Vec<u8>,
    x_size: i16,
    y_size: i16,
    z_size: i16,
}

impl LevelFinalizePacket {
    pub fn new(x_size: i16, y_size: i16, z_size: i16) -> Self {
        Self {
            data: Vec::new(),
            x_size,
            y_size,
            z_size,
        }
    }
}

#[async_trait]
impl PacketTrait for LevelFinalizePacket {
    fn packet_id(&self) -> u8 {
        return 0x04;
    }
    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_short(self.x_size);
        writer.write_short(self.y_size);
        writer.write_short(self.z_size);
        self.data = writer.to_bytes().clone();
    }
    fn read(&mut self, _reader: &mut PacketReader) {}
    async fn resolve(
        &self,
        socket: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;

        Ok(())
    }
}

/*
fn packet_id(&self) -> u8 {
    return 0x00;
}
fn write(&mut self, writer: &mut PacketWriter) {}
fn read(&mut self, reader: &mut PacketReader) {}
async fn resolve(&self, socket: &mut TcpStream) {
async fn resolve(
    &self,
    socket: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    socket.write_all(&self.data).await?;

    Ok(())
}
*/
