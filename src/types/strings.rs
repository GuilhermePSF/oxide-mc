use super::traits::{McRead, McWrite};
use super::varint::VarInt;
use crate::error::MineError;
use std::io::{Read, Write};

pub const DEFAULT_MAX_STRING_LENGTH: usize = 32767;

impl McRead for String {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        read_string_bounded(reader, DEFAULT_MAX_STRING_LENGTH)
    }
}

impl McWrite for String {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        let bytes = self.as_bytes();
        VarInt(bytes.len() as i32).write(writer)?;
        writer.write_all(bytes)?;
        Ok(())
    }
}

impl McWrite for &str {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        let bytes = self.as_bytes();
        VarInt(bytes.len() as i32).write(writer)?;
        writer.write_all(bytes)?;
        Ok(())
    }
}

pub fn read_string_bounded<R: Read>(reader: &mut R, max_chars: usize) -> Result<String, MineError> {
    let byte_len = VarInt::read(reader)?.0;
    if byte_len < 0 {
        return Err(MineError::Protocol(format!(
            "Negative string byte length: {byte_len}"
        )));
    }

    let byte_len = byte_len as usize;
    // UTF-8 characters are at most 4 bytes each
    let max_bytes = max_chars.saturating_mul(4);
    if byte_len > max_bytes {
        return Err(MineError::StringTooLong {
            length: byte_len,
            max: max_bytes,
        });
    }

    let mut buf = vec![0u8; byte_len];
    reader.read_exact(&mut buf)?;

    let s = String::from_utf8(buf)?;
    if s.chars().count() > max_chars {
        return Err(MineError::StringTooLong {
            length: s.chars().count(),
            max: max_chars,
        });
    }

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_string_roundtrip() {
        let test_strings = vec![
            "",
            "localhost",
            "Player123",
            "A server MOTD with color codes §a§lHello!",
            "Emojis: 🌍🎮🔥",
        ];

        for original in test_strings {
            let mut buf = Vec::new();
            original.write(&mut buf).unwrap();

            let mut cursor = Cursor::new(buf);
            let decoded = String::read(&mut cursor).unwrap();
            assert_eq!(decoded, original);
        }
    }
}
