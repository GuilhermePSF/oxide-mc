use std::io::ErrorKind;
use std::net::TcpStream;
use std::time::{Duration, Instant};

use crate::error::MineError;
use crate::packet::handshake::ClientHandshakePacket;
use crate::packet::login::{ClientLoginStartPacket, ServerLoginSuccessPacket};
use crate::packet::play::{
    ServerKeepAlivePacket, ServerJoinGamePacket, ServerPlayerPositionAndLookPacket,
};
use crate::packet::status::{
    ServerPongPacket, ServerStatusResponsePacket, ClientPingPacket,
    ClientStatusRequestPacket,
};
use crate::packet::{Packet, RawPacket};

/// The lifecycle state of a client connection in the Minecraft protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Play,
    Closed,
}

/// Represents a single connected Minecraft client/player.
pub struct Connection {
    stream: TcpStream,
    pub state: ConnectionState,
    pub username: Option<String>,
    pub entity_id: i32,
    last_keep_alive: Instant,
    current_keep_alive_id: i64,
}

impl Connection {
    /// Initializes a new connection in the initial `Handshaking` state.
    pub fn new(stream: TcpStream, entity_id: i32) -> Self {
        Self {
            stream,
            state: ConnectionState::Handshaking,
            username: None,
            entity_id,
            last_keep_alive: Instant::now(),
            current_keep_alive_id: 1000,
        }
    }

    /// Handles the connection lifecycle through the protocol state machine until disconnection.
    pub fn handle(&mut self) -> Result<(), MineError> {
        while self.state != ConnectionState::Closed {
            match self.state {
                ConnectionState::Handshaking => self.handle_handshaking()?,
                ConnectionState::Status => self.handle_status()?,
                ConnectionState::Login => self.handle_login()?,
                ConnectionState::Play => self.handle_play()?,
                ConnectionState::Closed => break,
            }
        }
        Ok(())
    }

    fn handle_handshaking(&mut self) -> Result<(), MineError> {
        let raw = RawPacket::read(&mut self.stream)?;
        let handshake = ClientHandshakePacket::from_raw(&raw)?;

        println!(
            "[{:?}] Handshake received: proto={}, host={}:{}, next_state={}",
            self.stream.peer_addr().ok(),
            *handshake.protocol_version,
            handshake.server_address,
            handshake.server_port,
            *handshake.next_state
        );

        match *handshake.next_state {
            1 => self.state = ConnectionState::Status,
            2 => self.state = ConnectionState::Login,
            other => {
                eprintln!("Invalid next_state requested: {other}");
                self.state = ConnectionState::Closed;
            }
        }

        Ok(())
    }

    fn handle_status(&mut self) -> Result<(), MineError> {
        loop {
            let raw = match RawPacket::read(&mut self.stream) {
                Ok(pkt) => pkt,
                Err(MineError::UnexpectedEof) | Err(MineError::Io(_)) => {
                    self.state = ConnectionState::Closed;
                    break;
                }
                Err(e) => return Err(e),
            };

            match raw.id {
                ClientStatusRequestPacket::PACKET_ID => {
                    let response = ServerStatusResponsePacket::create_default(
                        // the weird symbols are the way Minecraft puts text with colors and styles
                        "§a§lRust 1.12.2 Server §r- Online!",
                        20,
                        0,
                    );
                    response.write_framed(&mut self.stream)?;
                }
                ClientPingPacket::PACKET_ID => {
                    let ping = ClientPingPacket::from_raw(&raw)?;
                    let pong = ServerPongPacket {
                        payload: ping.payload,
                    };
                    pong.write_framed(&mut self.stream)?;
                    // The client closes the connection after receiving Pong
                    self.state = ConnectionState::Closed;
                    break;
                }
                other => {
                    println!("Unknown status packet ID: 0x{other:02X}");
                    self.state = ConnectionState::Closed;
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_login(&mut self) -> Result<(), MineError> {
        let raw = RawPacket::read(&mut self.stream)?;
        if raw.id != ClientLoginStartPacket::PACKET_ID {
            return Err(MineError::InvalidPacketId(raw.id));
        }

        let login_start = ClientLoginStartPacket::from_raw(&raw)?;
        let username = login_start.username;
        println!(
            "[{:?}] Player logging in: {}",
            self.stream.peer_addr().ok(),
            username
        );

        let login_success = ServerLoginSuccessPacket::generate_offline(&username);
        login_success.write_framed(&mut self.stream)?;

        self.username = Some(username);
        self.state = ConnectionState::Play;
        Ok(())
    }

    fn handle_play(&mut self) -> Result<(), MineError> {
        let player_name = self.username.clone().unwrap_or_else(|| "Player".into());
        println!(
            "[{:?}] Entering Play state for {player_name} (Entity ID: {})",
            self.stream.peer_addr().ok(),
            self.entity_id
        );

        let join_game = ServerJoinGamePacket::default_spawn(self.entity_id);
        join_game.write_framed(&mut self.stream)?;

        let position = ServerPlayerPositionAndLookPacket::spawn_at(0.0, 64.0, 0.0, 1);
        position.write_framed(&mut self.stream)?;

        self.stream
            .set_read_timeout(Some(Duration::from_millis(500)))?;

        self.last_keep_alive = Instant::now();

        while self.state == ConnectionState::Play {
            if self.last_keep_alive.elapsed() >= Duration::from_secs(5) {
                self.current_keep_alive_id += 1;
                let keep_alive = ServerKeepAlivePacket {
                    keep_alive_id: self.current_keep_alive_id,
                };
                if let Err(e) = keep_alive.write_framed(&mut self.stream) {
                    println!("[{player_name}] Failed to send Keep Alive: {e}");
                    self.state = ConnectionState::Closed;
                    break;
                }
                self.last_keep_alive = Instant::now();
            }

            match RawPacket::read(&mut self.stream) {
                Ok(packet) => {
                    self.handle_incoming_play_packet(&packet)?;
                }
                Err(MineError::Io(err))
                    if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut =>
                {
                    continue;
                }
                Err(MineError::UnexpectedEof) => {
                    println!("[{player_name}] Disconnected (EOF)");
                    self.state = ConnectionState::Closed;
                    break;
                }
                Err(MineError::Io(err))
                    if err.kind() == ErrorKind::ConnectionReset
                        || err.kind() == ErrorKind::ConnectionAborted =>
                {
                    println!("[{player_name}] Connection reset by client");
                    self.state = ConnectionState::Closed;
                    break;
                }
                Err(err) => {
                    eprintln!("[{player_name}] Error reading play packet: {err}");
                    self.state = ConnectionState::Closed;
                    break;
                }
            }
        }

        println!("[{player_name}] Session ended.");
        Ok(())
    }

    /// Handles or safely drains Server packets in the Play state.
    fn handle_incoming_play_packet(&mut self, packet: &RawPacket) -> Result<(), MineError> {
        match packet.id {
            0x00 => {
                // Teleport Confirm
                // Acknowledged by client after PlayerPositionAndLook
            }
            0x0B => {
                // Keep Alive response from client
            }
            0x04 => {
                // Client Settings (locale, render distance, chat flags, etc.)
            }
            0x09 => {
                // Plugin Message / Custom Payload (e.g. MC|Brand)
            }
            0x0C | 0x0D | 0x0E | 0x0F => {
                // Player movement packets (Player, Position, Look, PositionAndLook)
            }
            other => {
                // Safely drain any unhandled client packet so the stream doesn't desync
                // (e.g. animation, inventory click, etc.)
                let _ = other;
            }
        }
        Ok(())
    }
}
