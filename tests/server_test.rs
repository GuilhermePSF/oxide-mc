use mine::packet::handshake::ClientHandshakePacket;
use mine::packet::login::{ServerLoginSuccessPacket, ClientLoginStartPacket};
use mine::packet::play::{
    ServerKeepAlivePacket, ServerJoinGamePacket, ServerPlayerPositionAndLookPacket,
    ClientTeleportConfirmPacket,
};
use mine::packet::status::{
    ServerPongPacket, ServerStatusResponsePacket, ClientPingPacket,
    ClientStatusRequestPacket,
};
use mine::packet::{Packet, RawPacket};
use mine::server::Server;
use mine::types::VarInt;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[test]
fn test_integration_full_server_list_ping_lifecycle() {
    let port = 25575;
    let bind_addr = format!("127.0.0.1:{port}");

    let srv_addr = bind_addr.clone();
    thread::spawn(move || {
        let server = Server::new(srv_addr);
        let _ = server.run();
    });

    thread::sleep(Duration::from_millis(100));

    let mut stream = TcpStream::connect(&bind_addr).expect("Failed to connect to test server");

    let handshake = ClientHandshakePacket {
        protocol_version: VarInt(340),
        server_address: "127.0.0.1".to_string(),
        server_port: port,
        next_state: VarInt(1),
    };
    handshake.write_framed(&mut stream).unwrap();

    ClientStatusRequestPacket.write_framed(&mut stream).unwrap();

    let status_res = ServerStatusResponsePacket::read_framed(&mut stream).unwrap();
    assert!(status_res.json_response.contains("1.12.2"));
    assert!(status_res.json_response.contains("340"));

    let ping = ClientPingPacket {
        payload: 9988776655,
    };
    ping.write_framed(&mut stream).unwrap();

    let pong = ServerPongPacket::read_framed(&mut stream).unwrap();
    assert_eq!(pong.payload, 9988776655);
}

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

    let mut stream = TcpStream::connect(&bind_addr).expect("Failed to connect to test server");

    let handshake = ClientHandshakePacket {
        protocol_version: VarInt(340),
        server_address: "127.0.0.1".to_string(),
        server_port: port,
        next_state: VarInt(2),
    };
    handshake.write_framed(&mut stream).unwrap();

    let login_start = ClientLoginStartPacket {
        username: "RustLearner".to_string(),
    };
    login_start.write_framed(&mut stream).unwrap();

    let login_success = ServerLoginSuccessPacket::read_framed(&mut stream).unwrap();
    assert_eq!(login_success.username, "RustLearner");
    assert!(!login_success.uuid.is_empty());

    let raw_join = RawPacket::read(&mut stream).unwrap();
    assert_eq!(raw_join.id, 0x23);
    let join_game = ServerJoinGamePacket::from_raw(&raw_join).unwrap();
    assert_eq!(join_game.gamemode, 1);

    let raw_pos = RawPacket::read(&mut stream).unwrap();
    assert_eq!(raw_pos.id, 0x2F);
    let pos = ServerPlayerPositionAndLookPacket::from_raw(&raw_pos).unwrap();
    assert_eq!(*pos.teleport_id, 1);

    let teleport_confirm = ClientTeleportConfirmPacket {
        teleport_id: VarInt(1),
    };
    teleport_confirm.write_framed(&mut stream).unwrap();

    stream
        .set_read_timeout(Some(Duration::from_secs(7)))
        .unwrap();
    let raw_keep_alive = RawPacket::read(&mut stream).unwrap();
    assert_eq!(raw_keep_alive.id, 0x1F);
    let keep_alive = ServerKeepAlivePacket::from_raw(&raw_keep_alive).unwrap();
    assert!(keep_alive.keep_alive_id > 0);
}
