# Step 1: Laying the Groundwork & The Types Engine

We start by initializing the project:

```bash
cargo new --bin mine
cd mine
```

We configure `Cargo.toml` with both a library target (`src/lib.rs`) and a binary target (`src/main.rs`). This lets us write integration tests in `tests/` that import the crate directly:

```toml
[package]
name = "mine"
version = "0.1.0"
edition = "2024"

[lib]
name = "mine"
path = "src/lib.rs"

[[bin]]
name = "mine"
path = "src/main.rs"

[dependencies]
```

---

## 1. Creating the Error Types

In `src/error.rs`, we define `MineError`:

```rust
// src/error.rs
use std::fmt;
use std::io;
use std::string::FromUtf8Error;
use crate::types::VarIntError;

#[derive(Debug)]
pub enum MineError {
    Io(io::Error),
    VarInt(VarIntError),
    InvalidUtf8(FromUtf8Error),
    StringTooLong { length: usize, max: usize },
    InvalidPacketId(i32),
    UnexpectedEof,
    Protocol(String),
}

impl fmt::Display for MineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MineError::Io(err) => write!(f, "I/O error: {err}"),
            MineError::VarInt(err) => write!(f, "VarInt error: {err}"),
            MineError::InvalidUtf8(err) => write!(f, "Invalid UTF-8 string: {err}"),
            MineError::StringTooLong { length, max } => {
                write!(f, "String length ({length}) exceeded maximum allowed ({max})")
            }
            MineError::InvalidPacketId(id) => write!(f, "Unknown or invalid packet ID: 0x{id:02X}"),
            MineError::UnexpectedEof => write!(f, "Unexpected end of stream"),
            MineError::Protocol(msg) => write!(f, "Protocol error: {msg}"),
        }
    }
}

impl std::error::Error for MineError {}

impl From<io::Error> for MineError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            MineError::UnexpectedEof
        } else {
            MineError::Io(err)
        }
    }
}

impl From<VarIntError> for MineError {
    fn from(err: VarIntError) -> Self {
        match err {
            VarIntError::Io(io_err) => MineError::from(io_err),
            VarIntError::Incomplete => MineError::UnexpectedEof,
            other => MineError::VarInt(other),
        }
    }
}

impl From<FromUtf8Error> for MineError {
    fn from(err: FromUtf8Error) -> Self {
        MineError::InvalidUtf8(err)
    }
}
```

---

## 2. Defining Serialization Traits

In `src/types/traits.rs`, we define the `McRead` and `McWrite` traits:

```rust
// src/types/traits.rs
use std::io::{Read, Write};
use crate::error::MineError;

pub trait McRead: Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError>;
}

pub trait McWrite {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError>;
}
```

---

## 3. Building VarInt and VarLong

In `src/types/varint.rs`, we implement the LEB128 bitwise loops and the `VarInt` newtype wrapper:

```rust
// src/types/varint.rs
use std::fmt;
use std::io::{self, Read, Write};
use std::ops::{Deref, DerefMut};
use crate::error::MineError;
use super::traits::{McRead, McWrite};

#[derive(Debug)]
pub enum VarIntError {
    Incomplete,
    TooBig,
    Io(io::Error),
}

impl fmt::Display for VarIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarIntError::Incomplete => write!(f, "VarInt byte stream ended prematurely"),
            VarIntError::TooBig => write!(f, "VarInt exceeded maximum size limit"),
            VarIntError::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl std::error::Error for VarIntError {}

impl From<io::Error> for VarIntError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            VarIntError::Incomplete
        } else {
            VarIntError::Io(err)
        }
    }
}

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarInt(pub i32);

impl McRead for VarInt {
    fn read<R: Read>(reader: &mut R) -> Result<Self, MineError> {
        Ok(VarInt(read_varint(reader)?))
    }
}

impl McWrite for VarInt {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), MineError> {
        encode_varint(self.0, writer)?;
        Ok(())
    }
}

impl Deref for VarInt {
    type Target = i32;
    fn deref(&self) -> &Self::Target { &self.0 }
}
```

---

## 4. Testing against Protocol Vectors

We add a unit test in `src/types/varint.rs` to verify that our encoding matches the official specification:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_vectors() {
        let test_cases = [
            (0, &[0x00][..]),
            (1, &[0x01]),
            (127, &[0x7F]),
            (128, &[0x80, 0x01]),
            (25565, &[0xDD, 0xC7, 0x01]),
            (-1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
        ];

        for (val, expected_bytes) in test_cases {
            let mut buf = Vec::new();
            encode_varint(val, &mut buf).unwrap();
            assert_eq!(buf.as_slice(), expected_bytes);

            let (decoded, count) = decode_varint(&buf).unwrap();
            assert_eq!(decoded, val);
            assert_eq!(count, expected_bytes.len());
        }
    }
}
```

Run `cargo test` to verify:

```bash
cargo test
```
