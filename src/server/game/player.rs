use std::sync::Arc;

use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};

use crate::server::network::packet::PacketTrait;

#[derive(Clone, Debug)]
pub struct Player {
    id: i8,
    name: String,
    current_world: String,
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub yaw: u8,
    pub pitch: u8,
    pub socket: Arc<Mutex<WriteHalf<TcpStream>>>,
}

impl Player {
    pub fn new(
        id: i8,
        name: String,
        current_world: String,
        socket: Arc<Mutex<WriteHalf<TcpStream>>>,
    ) -> Self {
        Self {
            id,
            name,
            current_world,
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
        &self.current_world
    }

    pub fn set_pos(&mut self, x: i16, y: i16, z: i16, pitch: u8, yaw: u8) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.pitch = pitch;
        self.yaw = yaw;
    }
}
