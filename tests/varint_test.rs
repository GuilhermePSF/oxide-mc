use mine::varint::{decode_varint, decode_varlong, encode_varint, encode_varlong};

#[test]
fn test_varint_roundtrip_integration() {
    let test_values = vec![0, 1, 127, 128, 255, 25565, 2097151, i32::MAX, -1, i32::MIN];

    for val in test_values {
        let mut buffer = Vec::new();
        encode_varint(val, &mut buffer).expect("Encoding failed");

        let (decoded, bytes_read) = decode_varint(&buffer).expect("Decoding failed");
        assert_eq!(decoded, val);
        assert_eq!(bytes_read, buffer.len());
    }
}

#[test]
fn test_varlong_roundtrip_integration() {
    let test_values = vec![0, 1, 127, 128, 255, i64::MAX, -1, i64::MIN];

    for val in test_values {
        let mut buffer = Vec::new();
        encode_varlong(val, &mut buffer).expect("Encoding failed");

        let (decoded, bytes_read) = decode_varlong(&buffer).expect("Decoding failed");
        assert_eq!(decoded, val);
        assert_eq!(bytes_read, buffer.len());
    }
}
