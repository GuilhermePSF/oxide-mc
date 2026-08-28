# The Minecraft Wire Format & VarInts

When I first opened the Minecraft protocol wiki on wiki.vg, one of the first things that caught my attention was seeing numbers typed as `VarInt` and `VarLong` instead of regular `i32` or `i64`.

If you serialize a standard 32-bit integer like `1`, it takes up four full bytes: `0x00 0x00 0x00 0x01`. In a multiplayer game, small numbers like packet IDs (`0`, `1`, `2`), small counts, and lengths are sent thousands of times per second. Sending three leading zero bytes for every small number wastes bandwidth.

To solve this, Minecraft uses variable-length integers based on LEB128 (Little-Endian Base 128), which is the same format used by Protocol Buffers and WebAssembly.

---

## How a VarInt Works

A `VarInt` uses between 1 and 5 bytes to represent a 32-bit signed integer (`i32`):
- Numbers from `0` to `127` take 1 byte
- Numbers up to `16,383` take 2 bytes
- Large positive numbers or negative numbers take up to 5 bytes

Every byte in a `VarInt` is divided into two sections:

```text
  Bit 7 (MSB)               Bits 6 .. 0
+-------------------------+-----------------------------------------+
| Continuation Bit (0x80) | 7 Bits of Actual Data Payload (0x7F)   |
+-------------------------+-----------------------------------------+
```

1. **The Continuation Bit (Bit 7, `0x80`)**:
   - `1`: More bytes follow for this number.
   - `0`: This is the final byte.
2. **The 7 Data Bits (Bits 0 to 6, `0x7F`)**:
   - Holds 7 bits of the value, stored in little-endian order (least significant 7 bits first).

### Tracing Port 25565

Here is how the default port `25565` converts into bytes:

1. Write `25565` in binary:
   `00000000 00000000 01100011 11011101`
2. Group it into 7-bit chunks starting from the right:
   - Chunk 1: `1011101` (decimal 93, hex `0x5D`)
   - Chunk 2: `1000111` (decimal 71, hex `0x47`)
   - Chunk 3: `0000001` (decimal 1, hex `0x01`)
3. Set bit 7 on every chunk except the last:
   - Chunk 1 has more bytes after it: `1011101 | 0x80` = `11011101` (`0xDD`)
   - Chunk 2 has more bytes after it: `1000111 | 0x80` = `11000111` (`0xC7`)
   - Chunk 3 is the last chunk: leave MSB as 0 (`0x01`)
4. Output bytes: `[ 0xDD, 0xC7, 0x01 ]` (3 bytes instead of 4).

| Chunk | 7-Bit Data (Bits 0..6) | More Chunks? | MSB Continuation (Bit 7) | Combined 8-Bit Byte | Wire Hex |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Chunk 1** (least significant) | `1011101` (`93`) | Yes | `1` (`0x80`) | `11011101` | `0xDD` |
| **Chunk 2** | `1000111` (`71`) | Yes | `1` (`0x80`) | `11000111` | `0xC7` |
| **Chunk 3** (most significant) | `0000001` (`1`) | No (final) | `0` (`0x00`) | `00000001` | `0x01` |

### Test Vectors

| Decimal Value | Raw Hex | Wire Bytes |
|---|---|---|
| `0` | `0x00000000` | `0x00` (1 byte) |
| `1` | `0x00000001` | `0x01` (1 byte) |
| `127` | `0x0000007F` | `0x7F` (1 byte) |
| `128` | `0x00000080` | `0x80 0x01` (2 bytes) |
| `25565` | `0x000063DD` | `0xDD 0xC7 0x01` (3 bytes) |
| `2147483647` (`i32::MAX`) | `0x7FFFFFFF` | `0xFF 0xFF 0xFF 0xFF 0x07` (5 bytes) |
| `-1` | `0xFFFFFFFF` | `0xFF 0xFF 0xFF 0xFF 0x0F` (5 bytes) |

> Note on negative numbers: Because `i32` in Rust uses two's complement, `-1` has all 32 bits set to `1`. Since all bits are set, it takes the full 5 bytes across 7-bit chunks.

---

## The VarInt Encoder and Decoder

Here is the encoder in `src/types/varint.rs`:

```rust
pub fn encode_varint<W: std::io::Write>(value: i32, writer: &mut W) -> std::io::Result<usize> {
    let mut temp = value as u32;
    let mut bytes_written = 0;

    loop {
        let mut byte = (temp & 0x7F) as u8;
        temp >>= 7;
        if temp != 0 {
            byte |= 0x80; // Set continuation bit
        }
        writer.write_all(&[byte])?;
        bytes_written += 1;
        if temp == 0 {
            break;
        }
    }

    Ok(bytes_written)
}
```

And the decoder:

```rust
pub fn read_varint<R: std::io::Read>(reader: &mut R) -> Result<i32, VarIntError> {
    let mut value: i32 = 0;
    let mut shift: u32 = 0;
    let mut byte_buf = [0u8; 1];

    loop {
        reader.read_exact(&mut byte_buf)?;
        let byte = byte_buf[0];
        let val = (byte & 0x7F) as i32;
        value |= val << shift;

        // If MSB is 0, we reached the final byte
        if (byte & 0x80) == 0 {
            return Ok(value);
        }

        shift += 7;
        if shift >= 35 {
            return Err(VarIntError::TooBig);
        }
    }
}
```

---

## Fixed-Width Primitives (Big-Endian)

Fields like coordinates (`f64`), pitch/yaw (`f32`), entity IDs (`i32`), and port numbers (`u16`) use standard fixed-width binary encoding.

Minecraft uses Big-Endian (Network Byte Order) for all fixed primitives. In Big-Endian, the most significant byte comes first. Rust supports this directly with `.to_be_bytes()` and `from_be_bytes()`:

```rust
// Writing a Big-Endian u16 port:
writer.write_all(&port.to_be_bytes())?;

// Reading a Big-Endian u16 port:
let mut buf = [0u8; 2];
reader.read_exact(&mut buf)?;
let port = u16::from_be_bytes(buf);
```

---

## Minecraft Strings

In Minecraft, a string is a UTF-8 byte array prefixed by a `VarInt` length header:

```text
+-----------------------------------+-----------------------------------+
|  Length in Bytes (VarInt)         |  UTF-8 Encoded String Bytes       |
+-----------------------------------+-----------------------------------+
```

Our helper `read_string_bounded` verifies that string lengths stay within reasonable limits:

```rust
pub fn read_string_bounded<R: std::io::Read>(
    reader: &mut R,
    max_chars: usize,
) -> Result<String, MineError> {
    let byte_len = VarInt::read(reader)?.0;
    if byte_len < 0 {
        return Err(MineError::Protocol(format!("Negative string length: {byte_len}")));
    }

    let byte_len = byte_len as usize;
    let max_bytes = max_chars.saturating_mul(4);
    if byte_len > max_bytes {
        return Err(MineError::StringTooLong { length: byte_len, max: max_bytes });
    }

    let mut buf = vec![0u8; byte_len];
    reader.read_exact(&mut buf)?;

    let s = String::from_utf8(buf)?;
    if s.chars().count() > max_chars {
        return Err(MineError::StringTooLong { length: s.chars().count(), max: max_chars });
    }

    Ok(s)
}
```
