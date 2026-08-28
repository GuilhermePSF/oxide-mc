# Error Handling & Protocol Invariants

When working with network sockets, things fail all the time.

Clients close windows abruptly, Wi-Fi drops, packets get cut in half, or an invalid packet ID arrives. If an error panics on a worker thread, that thread dies.

To handle errors cleanly, we use a single `MineError` enum in `src/error.rs` and convert underlying IO and parsing errors with Rust's `From` trait.

---

## The MineError Enum

In `src/error.rs`, we define the possible error cases:

```rust
#[derive(Debug)]
pub enum MineError {
    Io(std::io::Error),
    VarInt(VarIntError),
    InvalidUtf8(std::string::FromUtf8Error),
    StringTooLong { length: usize, max: usize },
    InvalidPacketId(i32),
    UnexpectedEof,
    Protocol(String),
}
```

- `Io`: Socket errors like connection resets, timeouts, and broken pipes.
- `VarInt`: Invalid variable-length integers (exceeding 5 bytes or stream truncated).
- `InvalidUtf8`: String payloads that cannot be decoded as valid UTF-8.
- `StringTooLong`: Usernames or strings that exceed maximum allowed character limits.
- `UnexpectedEof`: The socket closed cleanly before the expected number of bytes arrived.
- `Protocol`: Custom protocol violations, like packets exceeding 2 MB.

---

## Converting Errors with the From Trait

To let us use the `?` operator everywhere, we implement `From` for standard IO, VarInt, and UTF-8 errors:

```rust
impl From<std::io::Error> for MineError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::UnexpectedEof {
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

impl From<std::string::FromUtf8Error> for MineError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        MineError::InvalidUtf8(err)
    }
}
```

### Why We Normalize UnexpectedEof
When a player disconnects, the client closes its TCP socket and the server receives an EOF.

By converting both `io::ErrorKind::UnexpectedEof` and `VarIntError::Incomplete` into `MineError::UnexpectedEof`, our connection loop knows the difference between:
- A player normally leaving the game (we log "Session ended" and exit the thread loop)
- An unexpected socket crash or malformed data

---

## Implementing Display and Error

We implement `fmt::Display` and `std::error::Error` so `MineError` prints readable messages:

```rust
impl std::fmt::Display for MineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
```
