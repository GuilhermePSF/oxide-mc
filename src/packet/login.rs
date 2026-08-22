use crate::error::MineError;
use crate::packet::Packet;
use crate::types::{McRead, McWrite, read_string_bounded};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLoginStartPacket {
    pub username: String,
}

impl Packet for ClientLoginStartPacket {
    const PACKET_ID: i32 = 0x00;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let username = read_string_bounded(reader, 16)?;
        Ok(Self { username })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.username.write(writer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLoginSuccessPacket {
    pub uuid: String,
    pub username: String,
}

impl ServerLoginSuccessPacket {
    pub fn new(uuid: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            uuid: uuid.into(),
            username: username.into(),
        }
    }

    /// Generates a standard offline-mode UUID formatted with hyphens from a username.
    /// (code robbed from a random ahh repo on github, no idea on collisions ;P)
    pub fn generate_offline(username: &str) -> Self {
        // Generate a deterministic (sure buddy) UUID for the offline player
        let mut hash: u128 = 0;
        for (i, b) in username.bytes().enumerate() {
            hash = hash.wrapping_add((b as u128) << ((i % 16) * 8));
        }
        let uuid_str = format!(
            "{:08x}-{:04x}-3{:03x}-8{:03x}-{:012x}",
            (hash >> 96) as u32,
            (hash >> 80) as u16,
            (hash >> 68) as u16 & 0x0FFF,
            (hash >> 56) as u16 & 0x0FFF,
            hash as u64 & 0xFFFFFFFFFFFF
        );

        Self::new(uuid_str, username)
    }
}

impl Packet for ServerLoginSuccessPacket {
    const PACKET_ID: i32 = 0x02;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let uuid = read_string_bounded(reader, 36)?;
        let username = read_string_bounded(reader, 16)?;
        Ok(Self { uuid, username })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.uuid.write(writer)?;
        self.username.write(writer)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLoginDisconnectPacket {
    pub reason_json: String,
}


impl ServerLoginDisconnectPacket {
    pub fn new(reason: &str) -> Self {
        let reason_json = format!(r#"{{"text":"{reason}"}}"#);
        Self { reason_json }
    }
}

impl Packet for ServerLoginDisconnectPacket {
    const PACKET_ID: i32 = 0x00;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let reason_json = String::read(reader)?;
        Ok(Self { reason_json })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.reason_json.write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_login_start_roundtrip() {
        let original = ClientLoginStartPacket {
            username: "Steve".to_string(),
        };

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = ClientLoginStartPacket::decode(&mut cursor).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_login_success_roundtrip() {
        let original = ServerLoginSuccessPacket::generate_offline("Alex");

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = ServerLoginSuccessPacket::decode(&mut cursor).unwrap();

        assert_eq!(decoded, original);
        assert_eq!(decoded.username, "Alex");
        assert_eq!(decoded.uuid.len(), 36);
    }
}
