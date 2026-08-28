# Testing the Protocol & Integration Verification

When I was first building this server, my testing workflow was pretty tedious: run `cargo run`, open the official Minecraft client, click Multiplayer, see if it connected, and check the terminal logs.

That works for quick manual checks, but it only tests the "happy path." What happens if a client disconnects halfway through a packet? What if packets arrive fragmented?

To make sure everything was solid without having to launch the game every time, I wrote automated **integration tests** in `tests/` that spin up our server on a real TCP socket and simulate client connections.

---

## 1. Testing Server List Ping Over Real TCP

In `tests/server_test.rs`, we spawn the server on a background thread, connect with a standard `TcpStream`, and test the full Server List Ping lifecycle:

```rust
// tests/server_test.rs
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use mine::packet::handshake::ClientHandshakePacket;
use mine::packet::status::{
    ServerPongPacket, ServerStatusResponsePacket, ClientPingPacket,
    ClientStatusRequestPacket,
};
use mine::packet::Packet;
use mine::server::Server;
use mine::types::VarInt;

#[test]
fn test_integration_full_server_list_ping_lifecycle() {
    let port = 25575; // Dedicated test port
    let bind_addr = format!("127.0.0.1:{port}");

    // 1. Spawn server in a background thread
    let srv_addr = bind_addr.clone();
    thread::spawn(move || {
        let server = Server::new(srv_addr);
        let _ = server.run();
    });

    thread::sleep(Duration::from_millis(100));

    // 2. Connect a client socket
    let mut stream = TcpStream::connect(&bind_addr).expect("Failed to connect to test server");

    // 3. Send Handshake (next_state = 1: Status)
    let handshake = ClientHandshakePacket {
        protocol_version: VarInt(340),
        server_address: "127.0.0.1".to_string(),
        server_port: port,
        next_state: VarInt(1),
    };
    handshake.write_framed(&mut stream).unwrap();

    // 4. Send Status Request
    ClientStatusRequestPacket.write_framed(&mut stream).unwrap();

    // 5. Verify the MOTD response contains our version
    let status_res = ServerStatusResponsePacket::read_framed(&mut stream).unwrap();
    assert!(status_res.json_response.contains("1.12.2"));
    assert!(status_res.json_response.contains("340"));

    // 6. Send Ping timestamp
    let ping = ClientPingPacket { payload: 9988776655 };
    ping.write_framed(&mut stream).unwrap();

    // 7. Verify Pong echoes the exact same timestamp
    let pong = ServerPongPacket::read_framed(&mut stream).unwrap();
    assert_eq!(pong.payload, 9988776655);
}
```

---

## 2. Testing the Login & World Spawning Flow

Next, we write a test simulating a player connecting and spawning into the world:

```rust
use mine::packet::login::{ServerLoginSuccessPacket, ClientLoginStartPacket};
use mine::packet::play::{
    ServerKeepAlivePacket, ServerJoinGamePacket, ServerPlayerPositionAndLookPacket,
    ClientTeleportConfirmPacket,
};
use mine::packet::RawPacket;

#[test]
fn test_integration_full_login_and_spawn_lifecycle() {
    let port = 25576;
    let bind_addr = format!("127.0.0.1:{port}");

    let srv_addr = bind_addr.clone();
    thread::spawn(move || {
        let server = Server::new(srv_addr);
        let _ = server.run();
    });

    thread::sleep(Duration::from_millis(100));

    let mut stream = TcpStream::connect(&bind_addr).expect("Failed to connect");

    // 1. Handshake with next_state = 2 (Login)
    ClientHandshakePacket {
        protocol_version: VarInt(340),
        server_address: "127.0.0.1".to_string(),
        server_port: port,
        next_state: VarInt(2),
    }.write_framed(&mut stream).unwrap();

    // 2. Send Login Start
    ClientLoginStartPacket {
        username: "RustLearner".to_string(),
    }.write_framed(&mut stream).unwrap();

    // 3. Receive Login Success
    let login_success = ServerLoginSuccessPacket::read_framed(&mut stream).unwrap();
    assert_eq!(login_success.username, "RustLearner");
    assert!(!login_success.uuid.is_empty());

    // 4. Receive Join Game (Play 0x23)
    let raw_join = RawPacket::read(&mut stream).unwrap();
    assert_eq!(raw_join.id, 0x23);
    let join_game = ServerJoinGamePacket::from_raw(&raw_join).unwrap();
    assert_eq!(join_game.gamemode, 1); // Creative mode

    // 5. Receive Player Position And Look (Play 0x2F)
    let raw_pos = RawPacket::read(&mut stream).unwrap();
    assert_eq!(raw_pos.id, 0x2F);

    // 6. Send Teleport Confirm acknowledgment
    ClientTeleportConfirmPacket { teleport_id: VarInt(1) }.write_framed(&mut stream).unwrap();

    // 7. Receive Keep-Alive heartbeat packet within ~5 seconds
    stream.set_read_timeout(Some(Duration::from_secs(7))).unwrap();
    let raw_keep_alive = RawPacket::read(&mut stream).unwrap();
    assert_eq!(raw_keep_alive.id, 0x1F);
}
```

---

## 3. Running the Tests

Running `cargo test` runs all our unit tests and integration tests:

```bash
cargo test
```

```text
running 17 tests
test packet::handshake::tests::test_handshake_roundtrip ... ok
test packet::login::tests::test_login_start_roundtrip ... ok
test packet::login::tests::test_login_success_roundtrip ... ok
test packet::play::tests::test_join_game_roundtrip ... ok
test packet::play::tests::test_keep_alive_roundtrip ... ok
test packet::play::tests::test_position_and_look_roundtrip ... ok
test packet::status::tests::test_ping_pong_roundtrip ... ok
test types::primitives::tests::test_primitives_roundtrip ... ok
test types::strings::tests::test_string_roundtrip ... ok
test types::varint::tests::test_protocol_varint_spec_examples ... ok
...
test test_integration_full_server_list_ping_lifecycle ... ok
test test_integration_full_login_and_spawn_lifecycle ... ok

test result: ok. 26 passed; 0 failed; finished in 5.23s
```

Seeing all 26 tests pass in a few seconds gives so much confidence that our binary codecs, packet framing, and state machine are doing exactly what they're supposed to do.
