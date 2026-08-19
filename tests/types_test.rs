use mine::error::MineError;
use mine::types::{McRead, McWrite, VarInt, VarLong, read_string_bounded};
use std::io::Cursor;

#[test]
fn test_integration_types_roundtrip() {
    let mut buffer = Vec::new();

    // Serialize a mix of data mimicking a Minecraft packet payload
    VarInt(340).write(&mut buffer).unwrap(); // Protocol version
    "localhost".to_string().write(&mut buffer).unwrap(); // Server address
    25565u16.write(&mut buffer).unwrap(); // Server port
    VarInt(2).write(&mut buffer).unwrap(); // Next state (Login)
    true.write(&mut buffer).unwrap(); // Boolean flag
    (-100.5f64).write(&mut buffer).unwrap(); // Position coordinate
    VarLong(123456789).write(&mut buffer).unwrap(); // Keep alive id

    // Deserialize
    let mut cursor = Cursor::new(buffer);
    let proto_version = VarInt::read(&mut cursor).unwrap();
    let host = String::read(&mut cursor).unwrap();
    let port = u16::read(&mut cursor).unwrap();
    let next_state = VarInt::read(&mut cursor).unwrap();
    let flag = bool::read(&mut cursor).unwrap();
    let coord = f64::read(&mut cursor).unwrap();
    let keep_alive = VarLong::read(&mut cursor).unwrap();

    assert_eq!(proto_version, VarInt(340));
    assert_eq!(*proto_version, 340);
    assert_eq!(host, "localhost");
    assert_eq!(port, 25565);
    assert_eq!(next_state, VarInt(2));
    assert_eq!(flag, true);
    assert_eq!(coord, -100.5);
    assert_eq!(keep_alive, VarLong(123456789));
}

#[test]
fn test_integration_string_length_limits() {
    let username = "VeryLongPlayerNameExceeding16Chars";
    let mut buffer = Vec::new();
    username.write(&mut buffer).unwrap();

    let mut cursor = Cursor::new(buffer);
    // Minecraft usernames are max 16 chars
    let res = read_string_bounded(&mut cursor, 16);
    match res {
        Err(MineError::StringTooLong { length, max }) => {
            assert_eq!(length, username.len());
            assert_eq!(max, 16);
        }
        _ => panic!("Expected StringTooLong error, got {res:?}"),
    }
}
