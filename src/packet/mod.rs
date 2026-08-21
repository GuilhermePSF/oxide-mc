use crate::error::MineError;
use crate::types::{encode_varint, read_varint, varint_size};
use std::io::{Cursor, Read, Write};

pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

pub use handshake::*;
pub use login::*;
pub use play::*;
pub use status::*;

pub const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPacket {
    pub id: i32,
    pub payload: Vec<u8>,
}

impl RawPacket {
    #[inline]
    pub fn new(id: i32, payload: Vec<u8>) -> Self {
        Self { id, payload }
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let packet_len = read_varint(reader)?;
        if packet_len <= 0 {
            return Err(MineError::Protocol(format!(
                "Invalid packet length: {packet_len}"
            )));
        }

        let packet_len = packet_len as usize;
        if packet_len > MAX_PACKET_SIZE {
            return Err(MineError::Protocol(format!(
                "Packet length ({packet_len} bytes) exceeds maximum permitted size ({MAX_PACKET_SIZE} bytes)"
            )));
        }

        let mut packet_buf = vec![0u8; packet_len];
        reader.read_exact(&mut packet_buf)?;

        let mut cursor = Cursor::new(&packet_buf);
        let id = read_varint(&mut cursor)?;
        let id_bytes_read = cursor.position() as usize;
        let payload = packet_buf[id_bytes_read..].to_vec();

        Ok(RawPacket { id, payload })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        let id_size = varint_size(self.id);
        let total_len = id_size + self.payload.len();

        encode_varint(total_len as i32, writer)?;
        encode_varint(self.id, writer)?;
        writer.write_all(&self.payload)?;
        writer.flush()?;

        Ok(())
    }

    #[inline]
    pub fn payload_cursor(&self) -> Cursor<&[u8]> {
        Cursor::new(&self.payload)
    }
}

pub trait Packet: Sized {
    const PACKET_ID: i32;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError>;
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MineError>;

    fn to_raw(&self) -> Result<RawPacket, MineError> {
        let mut payload = Vec::new();
        self.encode(&mut payload)?;
        Ok(RawPacket::new(Self::PACKET_ID, payload))
    }

    fn from_raw(raw: &RawPacket) -> Result<Self, MineError> {
        if raw.id != Self::PACKET_ID {
            return Err(MineError::InvalidPacketId(raw.id));
        }
        let mut cursor = raw.payload_cursor();
        Self::decode(&mut cursor)
    }

    fn write_framed<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        let raw = self.to_raw()?;
        raw.write(writer)
    }

    fn read_framed<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let raw = RawPacket::read(reader)?;
        Self::from_raw(&raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_packet_roundtrip() {
        let original_payload = vec![0x01, 0x02, 0x03, 0x04, 0xFF];
        let original_packet = RawPacket::new(0x02, original_payload.clone());

        let mut stream = Vec::new();
        original_packet.write(&mut stream).unwrap();

        let mut reader = Cursor::new(stream);
        let decoded_packet = RawPacket::read(&mut reader).unwrap();

        assert_eq!(decoded_packet.id, 0x02);
        assert_eq!(decoded_packet.payload, original_payload);
    }

    #[test]
    fn test_multiple_packets_in_stream() {
        let mut stream = Vec::new();

        // Write 3 consecutive packets into the same stream
        let p1 = RawPacket::new(0x00, vec![10, 20]);
        let p2 = RawPacket::new(0x01, vec![30, 40, 50]);
        let p3 = RawPacket::new(0x02, vec![60]);

        p1.write(&mut stream).unwrap();
        p2.write(&mut stream).unwrap();
        p3.write(&mut stream).unwrap();

        // Read them sequentially from the stream
        let mut reader = Cursor::new(stream);
        let r1 = RawPacket::read(&mut reader).unwrap();
        let r2 = RawPacket::read(&mut reader).unwrap();
        let r3 = RawPacket::read(&mut reader).unwrap();

        assert_eq!(r1, p1);
        assert_eq!(r2, p2);
        assert_eq!(r3, p3);
    }

    #[test]
    fn test_packet_size_limit_exceeded() {
        // Construct a fake header claiming packet is 5 MB (> 2 MB limit)
        let mut stream = Vec::new();
        encode_varint((5 * 1024 * 1024) as i32, &mut stream).unwrap();

        let mut reader = Cursor::new(stream);
        let res = RawPacket::read(&mut reader);
        assert!(matches!(res, Err(MineError::Protocol(_))));
    }
}
