use mine::error::MineError;
use mine::packet::{Packet, RawPacket};
use mine::types::{McRead, McWrite, VarInt};
use std::io::{Cursor, Read, Write};

#[derive(Debug, PartialEq, Eq)]
struct TestClientHandshakePacket {
    pub protocol_version: VarInt,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: VarInt,
}

impl Packet for TestClientHandshakePacket {
    const PACKET_ID: i32 = 0x00;

    fn decode<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        Ok(Self {
            protocol_version: VarInt::read(reader)?,
            server_address: String::read(reader)?,
            server_port: u16::read(reader)?,
            next_state: VarInt::read(reader)?,
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

#[test]
fn test_integration_typed_packet_framing() {
    let original = TestClientHandshakePacket {
        protocol_version: VarInt(340),
        server_address: "127.0.0.1".to_string(),
        server_port: 25565,
        next_state: VarInt(2),
    };

    let mut stream = Vec::new();
    original
        .write_framed(&mut stream)
        .expect("Writing framed packet failed");

    let mut reader = Cursor::new(stream);
    let decoded =
        TestClientHandshakePacket::read_framed(&mut reader).expect("Reading framed packet failed");

    assert_eq!(decoded, original);
}

#[test]
fn test_integration_stream_of_multiple_packets() {
    let p1 = RawPacket::new(0x00, vec![0xDE, 0xAD]);
    let p2 = RawPacket::new(0x01, vec![0xBE, 0xEF, 0xCA, 0xFE]);
    let p3 = RawPacket::new(0x02, vec![0x00]);

    let mut stream = Vec::new();
    p1.write(&mut stream).unwrap();
    p2.write(&mut stream).unwrap();
    p3.write(&mut stream).unwrap();

    let mut reader = Cursor::new(stream);
    assert_eq!(RawPacket::read(&mut reader).unwrap(), p1);
    assert_eq!(RawPacket::read(&mut reader).unwrap(), p2);
    assert_eq!(RawPacket::read(&mut reader).unwrap(), p3);
}

#[test]
fn test_integration_incomplete_stream_eof() {
    let p = RawPacket::new(0x05, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    let mut stream = Vec::new();
    p.write(&mut stream).unwrap();

    // Truncate the stream before the full payload is written
    let truncated = &stream[0..stream.len() - 3];
    let mut reader = Cursor::new(truncated);

    let res = RawPacket::read(&mut reader);
    assert!(matches!(res, Err(MineError::UnexpectedEof)));
}
