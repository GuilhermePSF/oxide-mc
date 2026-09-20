# Mine (oxide-mc)

> A lightweight, zero-dependency Minecraft 1.12.2 (Protocol #340) server built from scratch in Rust using only `std::net` raw TCP sockets and standard library primitives. Features a custom binary wire codec, connection state machine, and multi-client threading with thr purpose of learning rust, TCP and server design.

![Language](https://img.shields.io/badge/language-Rust-orange)

---

## Requirements

| Dependency | Notes |
| --- | --- |
| **Rust Toolchain** | `rustc` / `cargo` (Rust 2024 edition) |
| **Minecraft Client** | Java Edition 1.12.2 (Protocol 340) |
| **External Crates** | **None** — built strictly with `std` (no `tokio`, `serde`, `byteorder`, or `nom`) |

---

## Run

```bash
cargo run --release
```

By default, the server binds to dual-stack `[::]:25565` (supporting both IPv4 and IPv6) with automatic fallback to `0.0.0.0:25565`.

To specify a custom port:

```bash
PORT=25565 cargo run
```

Connect in Minecraft Java Edition 1.12.2 via `localhost:25565` or `127.0.0.1:25565`.

---

## Architecture

The codebase is organized into clean, modular layers:

- **`types`**: Zero-dependency binary wire encoding & decoding
  - `varint`: Minecraft variable-length integer (`VarInt` / `VarLong`) zig-zag-free format
  - `primitives`: Big-Endian numeric parsers (`i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `bool`)
  - `strings`: Length-prefixed UTF-8 string serialization
  - `traits`: `Encode` and `Decode` core traits
- **`packet`**: Framing and packet codec system
  - `RawPacket`: VarInt length-prefixed packet framing (`Packet Length + Packet ID + Payload`)
  - `Packet` trait: Strongly-typed encode/decode abstractions
  - Submodules: `handshake`, `status`, `login`, and `play`
- **`connection`**: State machine driving client socket lifecycles
  - Handles transitions: `Handshaking` $\rightarrow$ `Status` or `Login` $\rightarrow$ `Play`
- **`server`**: Concurrency and TCP listener
  - Multi-threaded dispatcher spawning dedicated worker threads per connection
  - Thread-safe entity ID distribution via `AtomicI32`
- **`world`**: Voxel terrain and chunk data structures
  - Block palettes, coordinates, chunk sections, bit storage, and generators (`flat`, `noise`)

---

## Protocol States & Packets

| State | Packet / Feature | Description |
| --- | --- | --- |
| **Handshake** | `Handshake` (`0x00`) | Identifies protocol version (340) and intends next state (`Status` or `Login`) |
| **Status** | `StatusRequest` (`0x00`) / `StatusResponse` (`0x00`) | Server List Ping with dynamic MOTD and online player count |
| | `Ping` (`0x01`) / `Pong` (`0x01`) | Client latency verification |
| **Login** | `LoginStart` (`0x00`) / `LoginSuccess` (`0x02`) | Offline-mode player authentication and UUID generation |
| **Play** | `JoinGame` (`0x23`) | Dispatches dimension, difficulty, entity ID, and game mode |
| | `SpawnPosition` (`0x46`) | Sets world spawn coordinates |
| | `PlayerPositionAndLook` (`0x2F`) | Teleports player and clears "Downloading Terrain" screen |
| | `PlayerAbilities` (`0x2C`) | Controls flight, creative, and invulnerability flags |
| | `KeepAlive` (`0x1F`) | Connection heartbeat tracking |

