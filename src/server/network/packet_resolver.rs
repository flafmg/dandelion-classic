use super::{
    packet::PacketTrait, packet_stream::packet_reader::PacketReader,
    packets::serverbound::PlayerIndentificationPacket,
};
use std::collections::HashMap;
use tokio::net::TcpStream;

pub struct PacketResolver {
    packet_handler: HashMap<u8, Box<dyn PacketTrait + Send + Sync>>,
}

impl PacketResolver {
    pub fn new() -> Self {
        let mut handler = HashMap::new();

        let player_packet: Box<dyn PacketTrait + Send + Sync> =
            Box::new(PlayerIndentificationPacket::new());
        handler.insert(player_packet.packet_id(), player_packet);

        Self {
            packet_handler: handler,
        }
    }

    pub async fn handle_packet(&mut self, data: &[u8], mut socket: TcpStream) {
        if data.is_empty() {
            println!("ERROR: empty data received");
            return;
        }

        let packet_id = data[0];

        if let Some(packet_trait) = self.packet_handler.get_mut(&packet_id) {
            let mut reader = PacketReader::new(data);
            packet_trait.read(&mut reader);
            packet_trait.resolve(&mut socket).await;
        } else {
            println!("ERROR: Unknown packet ID: {}", packet_id);
        }
    }
}
