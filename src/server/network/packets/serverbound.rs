use crate::server::network::{
    packet::PacketTrait,
    packet_stream::{packet_reader::PacketReader, packet_writer::PacketWriter},
};
use async_trait::async_trait;
use tokio::{io::AsyncWriteExt, net::TcpStream};

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
    async fn resolve(&self, socket: &mut TcpStream) {
        let mut resolve_packet_response = ServerIdentificationPacket::new();
        let mut paclet_writer = PacketWriter::new();
        resolve_packet_response.write(&mut paclet_writer);
        resolve_packet_response.resolve(socket).await;
    }
}

/* gonna copy'n paste

fn packet_id(&self) -> u8 {
    return 0x00;
}
fn write(&self, writer: &mut PacketWriter) {}
fn read(&self, reader: &mut PacketReader) {}
fn resolve(&self, socket: &mut TcpStream) {}

*/
