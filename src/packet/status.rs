use crate::error::MineError;
use crate::packet::Packet;
use crate::types::{McRead, McWrite};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientStatusRequestPacket;

impl Packet for ClientStatusRequestPacket {
    const PACKET_ID: i32 = 0x00;

    fn decode<R: Read>(_reader: &mut R) -> Result<Self, MineError> {
        Ok(Self)
    }

    fn encode<W: Write>(&self, _writer: &mut W) -> Result<(), MineError> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerStatusResponsePacket {
    pub json_response: String,
}

impl ServerStatusResponsePacket {
    pub fn new(json_response: impl Into<String>) -> Self {
        Self {
            json_response: json_response.into(),
        }
    }

    pub fn create_default(motd: &str, max_players: i32, online_players: i32) -> Self {
        let json = format!(
            r#"{{"version":{{"name":"1.12.2","protocol":340}},"players":{{"max":{max_players},"online":{online_players},"sample":[]}},"description":{{"text":"{motd}"}}}}"#
        );
        Self::new(json)
    }
}

impl Packet for ServerStatusResponsePacket {
    const PACKET_ID: i32 = 0x00;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let json_response = String::read(reader)?;
        Ok(Self { json_response })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.json_response.write(writer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPingPacket {
    pub payload: i64,
}

impl Packet for ClientPingPacket {
    const PACKET_ID: i32 = 0x01;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let payload = i64::read(reader)?;
        Ok(Self { payload })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.payload.write(writer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPongPacket {
    pub payload: i64,
}

impl Packet for ServerPongPacket {
    const PACKET_ID: i32 = 0x01;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let payload = i64::read(reader)?;
        Ok(Self { payload })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.payload.write(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_status_response_roundtrip() {
        let original = ServerStatusResponsePacket::create_default("A Rust Minecraft Server", 20, 0);

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = ServerStatusResponsePacket::decode(&mut cursor).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_ping_pong_roundtrip() {
        let ping = ClientPingPacket {
            payload: 123456789012345,
        };

        let mut buf = Vec::new();
        ping.encode(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let pong = ServerPongPacket::decode(&mut cursor).unwrap();

        assert_eq!(pong.payload, ping.payload);
    }
}
