use std::sync::Arc;

use async_trait::async_trait;
use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

use super::packet_stream::{packet_reader::PacketReader, packet_writer::PacketWriter};

#[async_trait]
pub trait PacketTrait: Send + Sync {
    fn packet_id(&self) -> u8;
    fn write(&mut self, writer: &mut PacketWriter);
    fn read(&mut self, reader: &mut PacketReader);
    async fn resolve(
        &self,
        socket: &mut WriteHalf<TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
