use crate::server::network::{
    packet::PacketTrait,
    packet_stream::{packet_reader::PacketReader, packet_writer::PacketWriter},
};
use async_trait::async_trait;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;

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
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;

        Ok(())
    }
}

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
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;

        Ok(())
    }
}

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
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

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
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;

        Ok(())
    }
}

pub struct PingPacket {
    data: Vec<u8>,
}
impl PingPacket {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}
#[async_trait]
impl PacketTrait for PingPacket {
    fn packet_id(&self) -> u8 {
        0x01
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;

        Ok(())
    }
}

pub struct SetBlockPacket {
    data: Vec<u8>,
    x: i16,
    y: i16,
    z: i16,
    block_type: u8,
}
impl SetBlockPacket {
    pub fn new(x: i16, y: i16, z: i16, block_type: u8) -> Self {
        Self {
            data: Vec::new(),
            x,
            y,
            z,
            block_type,
        }
    }
}
#[async_trait]
impl PacketTrait for SetBlockPacket {
    fn packet_id(&self) -> u8 {
        0x06
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_short(self.x);
        writer.write_short(self.y);
        writer.write_short(self.z);
        writer.write_byte(self.block_type);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct SpawnPlayerPacket {
    data: Vec<u8>,
    player_id: i8,
    player_name: String,
    x: i16,
    y: i16,
    z: i16,
    yaw: u8,
    pitch: u8,
}
impl SpawnPlayerPacket {
    pub fn new(
        player_id: i8,
        player_name: String,
        x: i16,
        y: i16,
        z: i16,
        yaw: u8,
        pitch: u8,
    ) -> Self {
        Self {
            data: Vec::new(),
            player_id,
            player_name,
            x,
            y,
            z,
            yaw,
            pitch,
        }
    }
}
#[async_trait]
impl PacketTrait for SpawnPlayerPacket {
    fn packet_id(&self) -> u8 {
        0x07
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_sbyte(self.player_id);
        writer.write_string(&self.player_name);
        writer.write_short(self.x);
        writer.write_short(self.y);
        writer.write_short(self.z);
        writer.write_byte(self.yaw);
        writer.write_byte(self.pitch);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct SetPositionAndOrientationPacket {
    data: Vec<u8>,
    player_id: i8,
    x: i16,
    y: i16,
    z: i16,
    yaw: u8,
    pitch: u8,
}
impl SetPositionAndOrientationPacket {
    pub fn new(player_id: i8, x: i16, y: i16, z: i16, yaw: u8, pitch: u8) -> Self {
        Self {
            data: Vec::new(),
            player_id,
            x,
            y,
            z,
            yaw,
            pitch,
        }
    }
}
#[async_trait]
impl PacketTrait for SetPositionAndOrientationPacket {
    fn packet_id(&self) -> u8 {
        0x08
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_sbyte(self.player_id);
        writer.write_short(self.x);
        writer.write_short(self.y);
        writer.write_short(self.z);
        writer.write_byte(self.yaw);
        writer.write_byte(self.pitch);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct PositionAndOrientationUpdatePacket {
    data: Vec<u8>,
    player_id: i8,
    delta_x: i8,
    delta_y: i8,
    delta_z: i8,
    yaw: u8,
    pitch: u8,
}
impl PositionAndOrientationUpdatePacket {
    pub fn new(player_id: i8, delta_x: i8, delta_y: i8, delta_z: i8, yaw: u8, pitch: u8) -> Self {
        Self {
            data: Vec::new(),
            player_id,
            delta_x,
            delta_y,
            delta_z,
            yaw,
            pitch,
        }
    }
}
#[async_trait]
impl PacketTrait for PositionAndOrientationUpdatePacket {
    fn packet_id(&self) -> u8 {
        0x09
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_sbyte(self.player_id);
        writer.write_byte(self.delta_x as u8);
        writer.write_byte(self.delta_y as u8);
        writer.write_byte(self.delta_z as u8);
        writer.write_byte(self.yaw);
        writer.write_byte(self.pitch);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct PositionUpdatePacket {
    data: Vec<u8>,
    player_id: i8,
    delta_x: i8,
    delta_y: i8,
    delta_z: i8,
}
impl PositionUpdatePacket {
    pub fn new(player_id: i8, delta_x: i8, delta_y: i8, delta_z: i8) -> Self {
        Self {
            data: Vec::new(),
            player_id,
            delta_x,
            delta_y,
            delta_z,
        }
    }
}
#[async_trait]
impl PacketTrait for PositionUpdatePacket {
    fn packet_id(&self) -> u8 {
        0x0a
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_sbyte(self.player_id);
        writer.write_byte(self.delta_x as u8);
        writer.write_byte(self.delta_y as u8);
        writer.write_byte(self.delta_z as u8);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct OrientationUpdatePacket {
    data: Vec<u8>,
    player_id: i8,
    yaw: u8,
    pitch: u8,
}
impl OrientationUpdatePacket {
    pub fn new(player_id: i8, yaw: u8, pitch: u8) -> Self {
        Self {
            data: Vec::new(),
            player_id,
            yaw,
            pitch,
        }
    }
}
#[async_trait]
impl PacketTrait for OrientationUpdatePacket {
    fn packet_id(&self) -> u8 {
        0x0b
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_sbyte(self.player_id);
        writer.write_byte(self.yaw);
        writer.write_byte(self.pitch);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct DespawnPlayerPacket {
    data: Vec<u8>,
    player_id: i8,
}
impl DespawnPlayerPacket {
    pub fn new(player_id: i8) -> Self {
        Self {
            data: Vec::new(),
            player_id,
        }
    }
}
#[async_trait]
impl PacketTrait for DespawnPlayerPacket {
    fn packet_id(&self) -> u8 {
        0x0c
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_sbyte(self.player_id);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct MessagePacket {
    data: Vec<u8>,
    player_id: i8,
    message: String,
}
impl MessagePacket {
    pub fn new(player_id: i8, message: String) -> Self {
        Self {
            data: Vec::new(),
            player_id,
            message,
        }
    }
}
#[async_trait]
impl PacketTrait for MessagePacket {
    fn packet_id(&self) -> u8 {
        0x0d
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_sbyte(self.player_id);
        writer.write_string(&self.message);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct DisconnectPlayerPacket {
    data: Vec<u8>,
    reason: String,
}
impl DisconnectPlayerPacket {
    pub fn new(reason: String) -> Self {
        Self {
            data: Vec::new(),
            reason,
        }
    }
}
#[async_trait]
impl PacketTrait for DisconnectPlayerPacket {
    fn packet_id(&self) -> u8 {
        0x0e
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_string(&self.reason);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}

pub struct UpdateUserTypePacket {
    data: Vec<u8>,
    user_type: u8,
}
impl UpdateUserTypePacket {
    pub fn new(user_type: u8) -> Self {
        Self {
            data: Vec::new(),
            user_type,
        }
    }
}
#[async_trait]
impl PacketTrait for UpdateUserTypePacket {
    fn packet_id(&self) -> u8 {
        0x0f
    }

    fn write(&mut self, writer: &mut PacketWriter) {
        writer.write_byte(self.packet_id());
        writer.write_byte(self.user_type);
        self.data = writer.to_bytes().clone();
    }

    fn read(&mut self, _reader: &mut PacketReader) {}

    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        socket.write_all(&self.data).await?;
        Ok(())
    }
}
