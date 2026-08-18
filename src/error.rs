use crate::types::VarIntError;
use std::fmt;
use std::io;
use std::string::FromUtf8Error;

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
                write!(
                    f,
                    "String length ({length}) exceeded maximum allowed ({max})"
                )
            }
            MineError::InvalidPacketId(id) => write!(f, "Unknown or invalid packet ID: 0x{id:02X}"),
            MineError::UnexpectedEof => write!(f, "Unexpected end of stream"),
            MineError::Protocol(msg) => write!(f, "Protocol error: {msg}"),
        }
    }
}

impl std::error::Error for MineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MineError::Io(err) => Some(err),
            MineError::VarInt(err) => Some(err),
            MineError::InvalidUtf8(err) => Some(err),
            _ => None,
        }
    }
}

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

impl From<&str> for MineError {
    fn from(msg: &str) -> Self {
        MineError::Protocol(msg.to_string())
    }
}

impl From<String> for MineError {
    fn from(msg: String) -> Self {
        MineError::Protocol(msg)
    }
}
