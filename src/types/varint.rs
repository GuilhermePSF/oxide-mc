use super::traits::{McRead, McWrite};
use crate::error::MineError;
use std::fmt;
use std::io::{self, Read, Write};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub enum VarIntError {
    Incomplete,
    TooBig,
    Io(io::Error),
}

impl fmt::Display for VarIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarIntError::Incomplete => {
                write!(f, "VarInt byte stream ended prematurely (incomplete)")
            }
            VarIntError::TooBig => write!(
                f,
                "VarInt exceeded maximum size (5 bytes for VarInt, 10 bytes for VarLong)"
            ),
            VarIntError::Io(err) => write!(f, "I/O error during VarInt processing: {err}"),
        }
    }
}

impl std::error::Error for VarIntError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VarIntError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for VarIntError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            VarIntError::Incomplete
        } else {
            VarIntError::Io(err)
        }
    }
}

pub const MAX_VARINT_BYTES: usize = 5;
pub const MAX_VARLONG_BYTES: usize = 10;

#[inline]
pub fn varint_size(value: i32) -> usize {
    let mut temp = value as u32;
    let mut count = 0;
    loop {
        count += 1;
        temp >>= 7;
        if temp == 0 {
            break;
        }
    }
    count
}

#[inline]
pub fn varlong_size(value: i64) -> usize {
    let mut temp = value as u64;
    let mut count = 0;
    loop {
        count += 1;
        temp >>= 7;
        if temp == 0 {
            break;
        }
    }
    count
}

pub fn encode_varint<W: Write>(value: i32, writer: &mut W) -> io::Result<usize> {
    let mut temp = value as u32;
    let mut bytes_written = 0;

    loop {
        let mut byte = (temp & 0x7F) as u8;
        temp >>= 7;
        if temp != 0 {
            byte |= 0x80;
        }
        writer.write_all(&[byte])?;
        bytes_written += 1;
        if temp == 0 {
            break;
        }
    }

    Ok(bytes_written)
}

pub fn decode_varint(bytes: &[u8]) -> Result<(i32, usize), VarIntError> {
    let mut value: i32 = 0;
    let mut shift: u32 = 0;
    let mut bytes_read: usize = 0;

    for &byte in bytes {
        bytes_read += 1;
        let val = (byte & 0x7F) as i32;
        value |= val << shift;

        if (byte & 0x80) == 0 {
            return Ok((value, bytes_read));
        }

        shift += 7;
        if shift >= 35 {
            return Err(VarIntError::TooBig);
        }
    }

    Err(VarIntError::Incomplete)
}

pub fn read_varint<R: Read>(reader: &mut R) -> Result<i32, VarIntError> {
    let mut value: i32 = 0;
    let mut shift: u32 = 0;
    let mut byte_buf = [0u8; 1];

    loop {
        reader.read_exact(&mut byte_buf)?;
        let byte = byte_buf[0];
        let val = (byte & 0x7F) as i32;
        value |= val << shift;

        if (byte & 0x80) == 0 {
            return Ok(value);
        }

        shift += 7;
        if shift >= 35 {
            return Err(VarIntError::TooBig);
        }
    }
}

#[inline]
pub fn write_varint<W: Write>(writer: &mut W, value: i32) -> Result<usize, VarIntError> {
    encode_varint(value, writer).map_err(VarIntError::from)
}

pub fn encode_varlong<W: Write>(value: i64, writer: &mut W) -> io::Result<usize> {
    let mut temp = value as u64;
    let mut bytes_written = 0;

    loop {
        let mut byte = (temp & 0x7F) as u8;
        temp >>= 7;
        if temp != 0 {
            byte |= 0x80;
        }
        writer.write_all(&[byte])?;
        bytes_written += 1;
        if temp == 0 {
            break;
        }
    }

    Ok(bytes_written)
}

pub fn decode_varlong(bytes: &[u8]) -> Result<(i64, usize), VarIntError> {
    let mut value: i64 = 0;
    let mut shift: u32 = 0;
    let mut bytes_read: usize = 0;

    for &byte in bytes {
        bytes_read += 1;
        let val = (byte & 0x7F) as i64;
        value |= val << shift;

        if (byte & 0x80) == 0 {
            return Ok((value, bytes_read));
        }

        shift += 7;
        if shift >= 70 {
            return Err(VarIntError::TooBig);
        }
    }

    Err(VarIntError::Incomplete)
}

pub fn read_varlong<R: Read>(reader: &mut R) -> Result<i64, VarIntError> {
    let mut value: i64 = 0;
    let mut shift: u32 = 0;
    let mut byte_buf = [0u8; 1];

    loop {
        reader.read_exact(&mut byte_buf)?;
        let byte = byte_buf[0];
        let val = (byte & 0x7F) as i64;
        value |= val << shift;

        if (byte & 0x80) == 0 {
            return Ok(value);
        }

        shift += 7;
        if shift >= 70 {
            return Err(VarIntError::TooBig);
        }
    }
}

#[inline]
pub fn write_varlong<W: Write>(writer: &mut W, value: i64) -> Result<usize, VarIntError> {
    encode_varlong(value, writer).map_err(VarIntError::from)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarInt(pub i32);

impl VarInt {
    #[inline]
    pub const fn new(val: i32) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for VarInt {
    #[inline]
    fn from(val: i32) -> Self {
        Self(val)
    }
}

impl From<VarInt> for i32 {
    #[inline]
    fn from(val: VarInt) -> Self {
        val.0
    }
}

impl Deref for VarInt {
    type Target = i32;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VarInt {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for VarInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl McRead for VarInt {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let val = read_varint(reader)?;
        Ok(VarInt(val))
    }
}

impl McWrite for VarInt {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        encode_varint(self.0, writer)?;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarLong(pub i64);

impl VarLong {
    #[inline]
    pub const fn new(val: i64) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl From<i64> for VarLong {
    #[inline]
    fn from(val: i64) -> Self {
        Self(val)
    }
}

impl From<VarLong> for i64 {
    #[inline]
    fn from(val: VarLong) -> Self {
        val.0
    }
}

impl Deref for VarLong {
    type Target = i64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VarLong {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for VarLong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl McRead for VarLong {
    #[inline]
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        let val = read_varlong(reader)?;
        Ok(VarLong(val))
    }
}

impl McWrite for VarLong {
    #[inline]
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        encode_varlong(self.0, writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_protocol_varint_spec_examples() {
        let cases: Vec<(i32, &[u8])> = vec![
            (0, &[0x00]),
            (1, &[0x01]),
            (2, &[0x02]),
            (127, &[0x7f]),
            (128, &[0x80, 0x01]),
            (255, &[0xff, 0x01]),
            (25565, &[0xdd, 0xc7, 0x01]),
            (2097151, &[0xff, 0xff, 0x7f]),
            (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
            (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
        ];

        for (val, expected_bytes) in cases {
            let mut buf = Vec::new();
            let written = encode_varint(val, &mut buf).unwrap();
            assert_eq!(buf.as_slice(), expected_bytes);
            assert_eq!(written, expected_bytes.len());
            assert_eq!(varint_size(val), expected_bytes.len());

            let (decoded, read_count) = decode_varint(&buf).unwrap();
            assert_eq!(decoded, val);
            assert_eq!(read_count, expected_bytes.len());

            let mut cursor = Cursor::new(&buf);
            let stream_decoded = read_varint(&mut cursor).unwrap();
            assert_eq!(stream_decoded, val);
        }
    }

    #[test]
    fn test_varint_incomplete() {
        let incomplete_buf = [0x80];
        let err = decode_varint(&incomplete_buf).unwrap_err();
        match err {
            VarIntError::Incomplete => {}
            _ => panic!("Expected Incomplete error, got {err:?}"),
        }
    }

    #[test]
    fn test_varint_too_big() {
        let too_big_buf = [0x80, 0x80, 0x80, 0x80, 0x80, 0x01];
        let err = decode_varint(&too_big_buf).unwrap_err();
        match err {
            VarIntError::TooBig => {}
            _ => panic!("Expected TooBig error, got {err:?}"),
        }
    }

    #[test]
    fn test_varlong_spec_examples() {
        let cases: Vec<(i64, &[u8])> = vec![
            (0, &[0x00]),
            (1, &[0x01]),
            (127, &[0x7f]),
            (128, &[0x80, 0x01]),
            (255, &[0xff, 0x01]),
            (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
            (
                9223372036854775807,
                &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
            ),
            (
                -1,
                &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
            ),
            (
                -9223372036854775808,
                &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
            ),
        ];

        for (val, expected_bytes) in cases {
            let mut buf = Vec::new();
            let written = encode_varlong(val, &mut buf).unwrap();
            assert_eq!(buf.as_slice(), expected_bytes);
            assert_eq!(written, expected_bytes.len());
            assert_eq!(varlong_size(val), expected_bytes.len());

            let (decoded, read_count) = decode_varlong(&buf).unwrap();
            assert_eq!(decoded, val);
            assert_eq!(read_count, expected_bytes.len());

            let mut cursor = Cursor::new(&buf);
            let stream_decoded = read_varlong(&mut cursor).unwrap();
            assert_eq!(stream_decoded, val);
        }
    }
}
