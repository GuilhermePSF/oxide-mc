use crate::error::MineError;
use crate::packet::Packet;
use crate::types::{McRead, McWrite, VarInt, read_string_bounded};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerJoinGamePacket {
    pub entity_id: i32,
    // 0 = Survival, 1 = Creative, 2 = Adventure, 3 = Spectator
    pub gamemode: u8,
    // -1 = Nether, 0 = Overworld, 1 = End
    pub dimension: i32,
    // 0 = Peaceful, 1 = Easy, 2 = Normal, 3 = Hard
    pub difficulty: u8,
    pub max_players: u8,
    pub level_type: String,
    pub reduced_debug_info: bool,
}


impl ServerJoinGamePacket {
    pub fn default_spawn(entity_id: i32) -> Self {
        Self {
            entity_id,
            gamemode: 1,   // Creative mode
            dimension: -1, // Nether
            difficulty: 1, // Easy
            max_players: 20,
            level_type: "default".to_string(),
            reduced_debug_info: false,
        }
    }
}

impl Packet for ServerJoinGamePacket {
    const PACKET_ID: i32 = 0x23;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let entity_id = i32::read(reader)?;
        let gamemode = u8::read(reader)?;
        let dimension = i32::read(reader)?;
        let difficulty = u8::read(reader)?;
        let max_players = u8::read(reader)?;
        let level_type = read_string_bounded(reader, 16)?;
        let reduced_debug_info = bool::read(reader)?;

        Ok(Self {
            entity_id,
            gamemode,
            dimension,
            difficulty,
            max_players,
            level_type,
            reduced_debug_info,
        })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.entity_id.write(writer)?;
        self.gamemode.write(writer)?;
        self.dimension.write(writer)?;
        self.difficulty.write(writer)?;
        self.max_players.write(writer)?;
        self.level_type.write(writer)?;
        self.reduced_debug_info.write(writer)?;
        Ok(())
    }
}

// Note: This packet is required for the client to exit the "Downloading Terrain" screen.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerPlayerPositionAndLookPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: i8,
    pub teleport_id: VarInt,
}


impl ServerPlayerPositionAndLookPacket {
    pub fn spawn_at(x: f64, y: f64, z: f64, teleport_id: i32) -> Self {
        Self {
            x,
            y,
            z,
            yaw: 0.0,
            pitch: 0.0,
            flags: 0,
            teleport_id: VarInt(teleport_id),
        }
    }
}

impl Packet for ServerPlayerPositionAndLookPacket {
    const PACKET_ID: i32 = 0x2F;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let x = f64::read(reader)?;
        let y = f64::read(reader)?;
        let z = f64::read(reader)?;
        let yaw = f32::read(reader)?;
        let pitch = f32::read(reader)?;
        let flags = i8::read(reader)?;
        let teleport_id = VarInt::read(reader)?;

        Ok(Self {
            x,
            y,
            z,
            yaw,
            pitch,
            flags,
            teleport_id,
        })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.x.write(writer)?;
        self.y.write(writer)?;
        self.z.write(writer)?;
        self.yaw.write(writer)?;
        self.pitch.write(writer)?;
        self.flags.write(writer)?;
        self.teleport_id.write(writer)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientTeleportConfirmPacket {
    pub teleport_id: VarInt,
}

impl Packet for ClientTeleportConfirmPacket {
    const PACKET_ID: i32 = 0x00;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let teleport_id = VarInt::read(reader)?;
        Ok(Self { teleport_id })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.teleport_id.write(writer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerKeepAlivePacket {
    pub keep_alive_id: i64,
}

impl Packet for ServerKeepAlivePacket {
    const PACKET_ID: i32 = 0x1F;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let keep_alive_id = i64::read(reader)?;
        Ok(Self { keep_alive_id })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.keep_alive_id.write(writer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientKeepAlivePacket {
    pub keep_alive_id: i64,
}

impl Packet for ClientKeepAlivePacket {
    const PACKET_ID: i32 = 0x0B;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let keep_alive_id = i64::read(reader)?;
        Ok(Self { keep_alive_id })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.keep_alive_id.write(writer)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientPlayerPositionPacket {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

impl Packet for ClientPlayerPositionPacket {
    const PACKET_ID: i32 = 0x0C;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let x = f64::read(reader)?;
        let y = f64::read(reader)?;
        let z = f64::read(reader)?;
        let on_ground = bool::read(reader)?;
        Ok(Self { x, y, z, on_ground })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.x.write(writer)?;
        self.y.write(writer)?;
        self.z.write(writer)?;
        self.on_ground.write(writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_join_game_roundtrip() {
        let original = ServerJoinGamePacket::default_spawn(1);

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = ServerJoinGamePacket::decode(&mut cursor).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_position_and_look_roundtrip() {
        let original = ServerPlayerPositionAndLookPacket::spawn_at(10.5, 64.0, -20.5, 1);

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = ServerPlayerPositionAndLookPacket::decode(&mut cursor).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_keep_alive_roundtrip() {
        let original = ServerKeepAlivePacket {
            keep_alive_id: 9876543210,
        };

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = ServerKeepAlivePacket::decode(&mut cursor).unwrap();

        assert_eq!(decoded, original);
    }
}
