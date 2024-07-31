use super::packet::PacketTrait;
use super::packet_stream::packet_reader::PacketReader;
use super::packet_stream::packet_writer::PacketWriter;
use super::packets::clientbound::{
    DespawnPlayerPacket, LevelDataChunkPacket, LevelFinalizePacket, LevelInitializePacket,
    PingPacket, UpdateSetBlockPacket,
};
use super::packets::serverbound::{
    MessagePacket, PlayerIndentificationPacket, PositionAndOrientationUpdatePacket, SetBlockPacket,
};
use crate::server::game::dmf_map::DmfMap;
use crate::server::game::player::Player;
use crate::server::network::packets::clientbound::{
    SendMessagePacket, SetPositionAndOrientationPacket, SpawnPlayerPacket,
};
use crate::server::server::Server;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::VecDeque;
use std::fmt::format;
use std::io::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{self, AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::sleep;

const PING_TIME_MILLIS: u64 = 100;
const PACKET_FLUSH_MILLIS: u64 = 20;

pub struct PacketQueue {
    queue: Arc<RwLock<VecDeque<(Option<i8>, Box<dyn PacketTrait>)>>>,
}

impl PacketQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn enqueue(&self, owner_id: Option<i8>, packet: Box<dyn PacketTrait>) {
        let mut queue = self.queue.write().await;
        queue.push_back((owner_id, packet));
    }

    pub async fn dequeue(&self) -> Option<(Option<i8>, Box<dyn PacketTrait>)> {
        let mut queue = self.queue.write().await;
        queue.pop_front()
    }
}

pub struct PacketResolver {
    pub server: Arc<Server>,
    pub packet_queue: PacketQueue,
}

impl PacketResolver {
    pub fn new(server: Arc<Server>) -> Self {
        Self {
            server,
            packet_queue: PacketQueue::new(),
        }
    }

    pub async fn handle_packet(&self, data: &[u8], socket: Arc<RwLock<WriteHalf<TcpStream>>>) {
        if data.is_empty() {
            eprintln!("Empty data received");
            return;
        }

        let packet_id = data[0];
        let mut reader = PacketReader::new(data);

        match packet_id {
            0x00 => self.player_connect(&mut reader, socket).await,
            0x05 => self.handle_set_block(&mut reader, socket).await,
            0x08 => {
                self.handle_position_and_orientation(&mut reader, socket)
                    .await
            }
            0x0d => self.handle_message(&mut reader, socket).await,
            _ => println!("Unknown packet ID: {}", packet_id),
        }
    }

    async fn player_connect(
        &self,
        reader: &mut PacketReader<'_>,
        socket: Arc<RwLock<WriteHalf<TcpStream>>>,
    ) {
        let packet = self.read_player_identification_packet(reader);
        let player_id = self.get_last_id().await;
        let player = self
            .create_player(player_id, packet.username.clone(), socket.clone())
            .await;
        self.add_player_to_server(player_id, player.clone()).await;
        self.send_server_identification(packet, player.socket.clone())
            .await;
        self.load_default_map_and_send_to_player(player.clone())
            .await;

        println!("{} connected", player.get_name());

        let join_message = format!("welcome {}!", player.get_name());
        for player in self.server.connected_players.iter() {
            player.send_message(&join_message).await;
        }

        player.send_message("&fwelcome to this silly server!").await;
        player
            .send_message("&edandelion &fis a &3server software&f in rust!")
            .await;
        player.send_message("&fcheck the source code bellow!").await;
        player
            .send_message("&bhttps://github.com/flafmg/dandelion-classic")
            .await;
        player
            .send_message("&cthis is indevelopment so it may be buggy")
            .await;
    }

    fn read_player_identification_packet(
        &self,
        reader: &mut PacketReader<'_>,
    ) -> PlayerIndentificationPacket {
        let mut packet = PlayerIndentificationPacket::new(
            self.server.config.name.clone(),
            self.server.config.motd.clone(),
        );
        packet.read(reader);
        packet
    }

    async fn create_player(
        &self,
        player_id: i8,
        username: String,
        socket: Arc<RwLock<WriteHalf<TcpStream>>>,
    ) -> Player {
        Player::new(
            player_id,
            username,
            "default".to_string(),
            Arc::clone(&socket),
        )
    }

    async fn add_player_to_server(&self, player_id: i8, player: Player) {
        self.server.connected_players.insert(player_id, player);
    }

    async fn send_server_identification(
        &self,
        packet: PlayerIndentificationPacket,
        socket: Arc<RwLock<WriteHalf<TcpStream>>>,
    ) {
        let resolve_result = {
            let mut socket = socket.write().await;
            packet.resolve(&mut socket).await
        };

        if let Err(e) = resolve_result {
            eprintln!("Error resolving server identification: {}", e);
        }
    }

    async fn load_default_map_and_send_to_player(&self, mut player: Player) {
        let default_map_name = self.server.config.default_map.clone();
        let map = self
            .server
            .loaded_maps
            .get(&default_map_name)
            .map(|e| e.clone());

        if let Some(mut map) = map {
            player.send_to_level(&map).await;
            map.set_spawn_point(4718, 1171, 4166);
            self.spawn_player(&mut player, &map).await;
        } else {
            eprintln!("Error: Default map '{}' not found.", default_map_name);
        }
    }

    async fn spawn_player(&self, player: &mut Player, map: &DmfMap) {
        player
            .teleport(map.x_spawn, map.y_spawn, map.z_spawn, 0, 0)
            .await;

        let spawn_player = SpawnPlayerPacket::new(
            player.get_id(),
            player.get_name().to_string(),
            player.x,
            player.y,
            player.z,
            0,
            0,
        );
        self.send_packet_to_all(Some(player), spawn_player).await;
        self.spawn_online_players(player).await;
    }

    async fn spawn_online_players(&self, new_player: &Player) {
        let mut socket = new_player.socket.write().await;
        for player in self.server.connected_players.iter() {
            if player.get_id() != new_player.get_id() {
                let mut spawn_player = SpawnPlayerPacket::new(
                    player.get_id(),
                    player.get_name().to_string(),
                    player.x,
                    player.y,
                    player.z,
                    player.yaw,
                    player.pitch,
                );
                let mut packet_writer = PacketWriter::new();
                spawn_player.write(&mut packet_writer);
                spawn_player.resolve(&mut socket).await;
            }
        }
    }

    async fn handle_set_block(
        &self,
        reader: &mut PacketReader<'_>,
        socket: Arc<RwLock<WriteHalf<TcpStream>>>,
    ) {
        let player = self
            .get_player_by_socket(Arc::clone(&socket))
            .await
            .unwrap();

        let mut set_block_packet = SetBlockPacket::new();
        set_block_packet.read(reader);

        let map_name = "default";
        let mut maps = self.server.loaded_maps.clone();
        let mut map = maps.get_mut(map_name).unwrap();
        let block = if set_block_packet.mode == 0x00 {
            0x00
        } else {
            set_block_packet.block_type
        };

        map.set_block(
            set_block_packet.x,
            set_block_packet.y,
            set_block_packet.z,
            block,
        );

        let update_set_block = UpdateSetBlockPacket::new(
            set_block_packet.x,
            set_block_packet.y,
            set_block_packet.z,
            block,
        );
        self.packet_queue
            .enqueue(Some(player.get_id()), Box::new(update_set_block))
            .await;
    }

    async fn handle_position_and_orientation(
        &self,
        reader: &mut PacketReader<'_>,
        socket: Arc<RwLock<WriteHalf<TcpStream>>>,
    ) {
        let mut player = self
            .get_player_by_socket(Arc::clone(&socket))
            .await
            .unwrap();

        let mut position_packet = PositionAndOrientationUpdatePacket::new();
        position_packet.read(reader);

        player.set_pos(
            position_packet.x,
            position_packet.y,
            position_packet.z,
            position_packet.pitch,
            position_packet.yaw,
        );

        let mut set_position_packet = SetPositionAndOrientationPacket::new(
            player.get_id(),
            player.x,
            player.y,
            player.z,
            player.yaw,
            player.pitch,
        );
        self.send_packet_to_all(Some(&player), set_position_packet)
            .await;
    }

    async fn handle_message(
        &self,
        reader: &mut PacketReader<'_>,
        socket: Arc<RwLock<WriteHalf<TcpStream>>>,
    ) {
        let mut player = self
            .get_player_by_socket(Arc::clone(&socket))
            .await
            .unwrap();

        let mut message_packet = MessagePacket::new();
        message_packet.read(reader);

        let message = format!("{}: {}", player.get_name(), message_packet.message);

        let mut send_message_packet = SendMessagePacket::new(player.get_id(), message);

        self.packet_queue
            .enqueue(None, Box::new(send_message_packet))
            .await;
    }

    async fn get_player_by_socket(
        &self,
        socket: Arc<RwLock<WriteHalf<TcpStream>>>,
    ) -> Option<Player> {
        for player in self.server.connected_players.iter() {
            if Arc::ptr_eq(&player.socket, &socket) {
                return Some(player.clone());
            }
        }
        None
    }

    pub async fn send_packet_to_all(&self, owner: Option<&Player>, mut packet: impl PacketTrait) {
        let mut packet_writer = PacketWriter::new();
        packet.write(&mut packet_writer);

        for player in self.server.connected_players.iter() {
            if let Some(owner) = owner {
                if player.get_id() == owner.get_id() {
                    continue;
                }
            }

            let mut socket = player.socket.write().await;
            packet.resolve(&mut socket).await;
        }
    }
    pub async fn despawn_player(&self, id: i8) {
        let despawn_player = DespawnPlayerPacket::new(id);
        self.packet_queue
            .enqueue(None, Box::new(despawn_player))
            .await;
    }
    pub async fn send_to_all_queued(&self) {
        loop {
            while let Some((owner_id, mut packet)) = self.packet_queue.dequeue().await {
                let mut packet_writer = PacketWriter::new();
                packet.write(&mut packet_writer);

                for player in self.server.connected_players.iter() {
                    if let Some(owner_id) = owner_id {
                        if player.get_id() == owner_id {
                            continue;
                        }
                    }

                    let mut socket = player.socket.write().await;
                    packet.resolve(&mut socket).await;
                }
            }
            sleep(Duration::from_millis(PACKET_FLUSH_MILLIS)).await;
        }
    }

    pub async fn ping_players_loop(&self) {
        loop {
            let mut remove_ids = Vec::new();
            for player in self.server.connected_players.iter() {
                let mut socket = player.socket.write().await;
                let mut ping_packet = PingPacket::new();
                let mut packet_writer = PacketWriter::new();
                ping_packet.write(&mut packet_writer);

                if let Err(er) = socket.write_all(&packet_writer.into_inner()).await {
                    println!("Player {} disconnected", player.get_name());
                    remove_ids.push(player.get_id());
                }
            }
            for id in remove_ids {
                let leave_message = format!(
                    "goodbye {}",
                    self.server.connected_players.get(&id).unwrap().get_name()
                );
                self.server.connected_players.remove(&id);
                self.despawn_player(id).await;

                for player in self.server.connected_players.iter() {
                    player.send_message(&leave_message).await;
                }
            }

            sleep(Duration::from_millis(PING_TIME_MILLIS)).await;
        }
    }

    async fn get_last_id(&self) -> i8 {
        for id in 0..=127 {
            let id = id as i8;
            if !self.server.connected_players.contains_key(&id) {
                return id;
            }
        }
        -1
    }
}
