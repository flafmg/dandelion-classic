use async_trait::async_trait;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::server::network::{
    packet::PacketTrait,
    packet_stream::{packet_reader::PacketReader, packet_writer::PacketWriter},
};

// server indentification packet

pub struct ServerIdentificationPacket {
    data: Vec<u8>,
    protocol_version: u8,
    server_name: String,
    server_motd: String,
    user_type: u8,
}
impl ServerIdentificationPacket {
    pub fn new() -> Self {
        return Self {
            data: Vec::new(),
            protocol_version: 0x07,
            server_name: "dandelion".to_string(),
            server_motd: "hello mom!".to_string(),
            user_type: 0x64,
        };
    }
}
#[async_trait]
impl PacketTrait for ServerIdentificationPacket {
    fn packet_id(&self) -> u8 {
        return 0x00;
    }
    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_byte(self.protocol_version);
        writer.write_string(self.server_name.as_str());
        writer.write_string(self.server_motd.as_str());
        writer.write_byte(self.user_type);
        self.data = writer.to_bytes().clone();
    }
    fn read(&mut self, reader: &mut PacketReader) {}

    async fn resolve(&self, socket: &mut TcpStream) {
        if socket.write_all(&self.data).await.is_err() {
            println!("Error sending packet id {}", self.packet_id())
        }
        let mut resolve_packet_response = LevelInitializePacket::new();
        let mut paclet_writer = PacketWriter::new();
        resolve_packet_response.write(&mut paclet_writer);
        resolve_packet_response.resolve(socket).await;
    }
}

// level initialize packet

pub struct LevelInitializePacket {
    data: Vec<u8>,
}
impl LevelInitializePacket {
    pub fn new() -> Self {
        return Self { data: Vec::new() };
    }
}
#[async_trait]
impl PacketTrait for LevelInitializePacket {
    fn packet_id(&self) -> u8 {
        return 0x02;
    }
    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        self.data = writer.to_bytes().clone();
    }
    fn read(&mut self, reader: &mut PacketReader) {}

    async fn resolve(&self, socket: &mut TcpStream) {
        if socket.write_all(&self.data).await.is_err() {
            println!("Error sending packet id {}", self.packet_id())
        }
        let mut resolve_packet_response = LevelFinalizePacket::new(100, 100, 100);
        let mut paclet_writer = PacketWriter::new();
        resolve_packet_response.write(&mut paclet_writer);
        resolve_packet_response.resolve(socket).await;
    }
}

// level data chunk packet

pub struct LevelDataChunkPacket {
    chunk_lenght: u16,
    chunk_data: Vec<u8>,
    percent_complete: u8,
}

impl LevelDataChunkPacket {
    pub fn new() -> Self {
        return Self {
            chunk_lenght: 0,
            chunk_data: Vec::new(),
            percent_complete: 0,
        };
    }
}

#[async_trait]
impl PacketTrait for LevelDataChunkPacket {
    fn packet_id(&self) -> u8 {
        return 0x03;
    }
    fn write(&mut self, writer: &mut PacketWriter) {}
    fn read(&mut self, reader: &mut PacketReader) {}
    async fn resolve(&self, socket: &mut TcpStream) {}
}

// leval finalize packet

pub struct LevelFinalizePacket {
    data: Vec<u8>,
    x_size: i16,
    y_size: i16,
    z_size: i16,
}

impl LevelFinalizePacket {
    pub fn new(x_size: i16, y_size: i16, z_size: i16) -> Self {
        return Self {
            data: Vec::new(),
            x_size,
            y_size,
            z_size,
        };
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
    fn read(&mut self, reader: &mut PacketReader) {}
    async fn resolve(&self, socket: &mut TcpStream) {
        if socket.write_all(&self.data).await.is_err() {
            println!("Error sending packet id {}", self.packet_id())
        }
    }
}
/*
fn packet_id(&self) -> u8 {
    return 0x00;
}
fn write(&mut self, writer: &mut PacketWriter) {}
fn read(&mut self, reader: &mut PacketReader) {}
async fn resolve(&self, socket: &mut TcpStream) {
async fn resolve(&self, socket: &mut TcpStream) {
    if socket.write_all(&self.data).await.is_err() {
        println!("Error sending packet id {}", self.packet_id())
    }
}

*/
