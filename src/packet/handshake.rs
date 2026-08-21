use crate::error::MineError;
use crate::packet::Packet;
use crate::types::{McRead, McWrite, VarInt, read_string_bounded};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientHandshakePacket {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: VarInt,
}


impl Packet for ClientHandshakePacket {
    const PACKET_ID: i32 = 0x00;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let protocol_version = VarInt::read(reader)?;
        let server_address = read_string_bounded(reader, 255)?;
        let server_port = u16::read(reader)?;
        let next_state = VarInt::read(reader)?;

        Ok(Self {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        self.protocol_version.write(writer)?;
        self.server_address.write(writer)?;
        self.server_port.write(writer)?;
        self.next_state.write(writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_handshake_roundtrip() {
        let original = ClientHandshakePacket {
            protocol_version: VarInt(340),
            server_address: "localhost".to_string(),
            server_port: 25565,
            next_state: VarInt(1),
        };

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = ClientHandshakePacket::decode(&mut cursor).unwrap();

        assert_eq!(decoded, original);
    }
}
