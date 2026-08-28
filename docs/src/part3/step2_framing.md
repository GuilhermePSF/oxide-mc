# Step 2: Packet Framing & The RawPacket Layer

With the types and codecs in place, we can implement packet framing using `RawPacket` and the `Packet` trait.

---

## 1. Creating RawPacket

In `src/packet/mod.rs`, we define `MAX_PACKET_SIZE` and create the `RawPacket` struct:

```rust
// src/packet/mod.rs
use std::io::{Cursor, Read, Write};
use crate::error::MineError;
use crate::types::{encode_varint, read_varint, varint_size};

pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

pub const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024; // 2 MB

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPacket {
    pub id: i32,
    pub payload: Vec<u8>,
}

impl RawPacket {
    pub fn new(id: i32, payload: Vec<u8>) -> Self {
        Self { id, payload }
    }

    /// Reads a single framed packet from any byte reader.
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let packet_len = read_varint(reader)?;
        if packet_len <= 0 {
            return Err(MineError::Protocol(format!("Invalid packet length: {packet_len}")));
        }

        let packet_len = packet_len as usize;
        if packet_len > MAX_PACKET_SIZE {
            return Err(MineError::Protocol(format!(
                "Packet size ({packet_len}) exceeds limit ({MAX_PACKET_SIZE})"
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

    /// Writes this framed packet to any byte writer.
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        let id_size = varint_size(self.id);
        let total_len = id_size + self.payload.len();

        encode_varint(total_len as i32, writer)?;
        encode_varint(self.id, writer)?;
        writer.write_all(&self.payload)?;
        writer.flush()?;

        Ok(())
    }

    pub fn payload_cursor(&self) -> Cursor<&[u8]> {
        Cursor::new(&self.payload)
    }
}
```

---

## 2. Defining the Packet Trait

In `src/packet/mod.rs`, we define the `Packet` trait:

```rust
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
        self.to_raw()?.write(writer)
    }

    fn read_framed<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let raw = RawPacket::read(reader)?;
        Self::from_raw(&raw)
    }
}
```

---

## 3. Testing Stream Coalescing

We write a test to make sure our framing reader correctly parses multiple packets sent back-to-back in a single buffer:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_coalescing() {
        let mut stream = Vec::new();

        let p1 = RawPacket::new(0x00, vec![1, 2, 3]);
        let p2 = RawPacket::new(0x01, vec![4, 5]);

        // Write both packets into one stream
        p1.write(&mut stream).unwrap();
        p2.write(&mut stream).unwrap();

        // Read them back sequentially
        let mut cursor = Cursor::new(stream);
        let read1 = RawPacket::read(&mut cursor).unwrap();
        let read2 = RawPacket::read(&mut cursor).unwrap();

        assert_eq!(read1, p1);
        assert_eq!(read2, p2);
    }
}
```
