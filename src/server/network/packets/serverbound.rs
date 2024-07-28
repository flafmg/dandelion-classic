use crate::server::network::{
    packet::PacketTrait,
    packet_stream::{packet_reader::PacketReader, packet_writer::PacketWriter},
};
use async_trait::async_trait;
use tokio::net::TcpStream;

use super::clientbound::ServerIdentificationPacket;

pub struct PlayerIndentificationPacket {
    protocol_version: u8,
    username: String,
    verification_key: String,
}
impl PlayerIndentificationPacket {
    pub fn new() -> Self {
        return Self {
            protocol_version: 0,
            username: String::new(),
            verification_key: String::new(),
        };
    }
}

#[async_trait]
impl PacketTrait for PlayerIndentificationPacket {
    fn packet_id(&self) -> u8 {
        return 0x00;
    }
    fn write(&mut self, writer: &mut PacketWriter) {}

    fn read(&mut self, reader: &mut PacketReader) {
        self.protocol_version = reader.read_byte();
        self.username = reader.read_string();
        self.verification_key = reader.read_string();

        println!(
            " username: {}\n protocol: {}",
            self.username, self.protocol_version
        )
    }
    async fn resolve(
        &self,
        socket: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut resolve_packet_response = ServerIdentificationPacket::new();
        let mut paclet_writer = PacketWriter::new();
        resolve_packet_response.write(&mut paclet_writer);
        resolve_packet_response.resolve(socket).await?;

        Ok(())
    }
}

pub struct SetBlockPacket {
    x: i16,
    y: i16,
    z: i16,
    mode: u8,
    block_type: u8,
}

impl SetBlockPacket {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            mode: 0,
            block_type: 0,
        }
    }
}

#[async_trait]
impl PacketTrait for SetBlockPacket {
    fn packet_id(&self) -> u8 {
        0x05
    }

    fn write(&mut self, _writer: &mut PacketWriter) {}

    fn read(&mut self, reader: &mut PacketReader) {
        self.x = reader.read_short();
        self.y = reader.read_short();
        self.z = reader.read_short();
        self.mode = reader.read_byte();
        self.block_type = reader.read_byte();

        println!(
            "SetBlockPacket: x={}, y={}, z={}, mode={}, block_type={}",
            self.x, self.y, self.z, self.mode, self.block_type
        );
    }

    async fn resolve(
        &self,
        _socket: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

pub struct PositionAndOrientationPacket {
    player_id: i8,
    x: i16,
    y: i16,
    z: i16,
    yaw: u8,
    pitch: u8,
}

impl PositionAndOrientationPacket {
    pub fn new() -> Self {
        Self {
            player_id: -1,
            x: 0,
            y: 0,
            z: 0,
            yaw: 0,
            pitch: 0,
        }
    }
}

#[async_trait]
impl PacketTrait for PositionAndOrientationPacket {
    fn packet_id(&self) -> u8 {
        0x08
    }

    fn write(&mut self, _writer: &mut PacketWriter) {}

    fn read(&mut self, reader: &mut PacketReader) {
        self.player_id = reader.read_sbyte();
        self.x = reader.read_short();
        self.y = reader.read_short();
        self.z = reader.read_short();
        self.yaw = reader.read_byte();
        self.pitch = reader.read_byte();

        println!(
            "PositionAndOrientationPacket: player_id={}, x={}, y={}, z={}, yaw={}, pitch={}",
            self.player_id, self.x, self.y, self.z, self.yaw, self.pitch
        );
    }

    async fn resolve(
        &self,
        _socket: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

pub struct MessagePacket {
    player_id: i8,
    message: String,
}

impl MessagePacket {
    pub fn new() -> Self {
        Self {
            player_id: -1,
            message: String::new(),
        }
    }
}

#[async_trait]
impl PacketTrait for MessagePacket {
    fn packet_id(&self) -> u8 {
        0x0d
    }

    fn write(&mut self, _writer: &mut PacketWriter) {}

    fn read(&mut self, reader: &mut PacketReader) {
        self.player_id = reader.read_sbyte();
        self.message = reader.read_string();

        println!(
            "MessagePacket: player_id={}, message={}",
            self.player_id, self.message
        );
    }

    async fn resolve(
        &self,
        _socket: &mut TcpStream,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
