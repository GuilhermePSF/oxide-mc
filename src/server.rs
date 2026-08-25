use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;

use crate::connection::Connection;
use crate::error::MineError;

pub struct Server {
    bind_address: String,
    entity_counter: Arc<AtomicI32>,
}

impl Server {
    pub fn new(bind_address: impl Into<String>) -> Self {
        Self {
            bind_address: bind_address.into(),
            entity_counter: Arc::new(AtomicI32::new(1)),
        }
    }

    pub fn run(&self) -> Result<(), MineError> {
        // Attempt dual-stack [::]:port first, fallback to 0.0.0.0 if IPv6 is disabled
        let listener = match TcpListener::bind(&self.bind_address) {
            Ok(l) => l,
            Err(err) => {
                if self.bind_address.starts_with("[::]") {
                    let port = self.bind_address.split(':').last().unwrap_or("25565");
                    let fallback = format!("0.0.0.0:{port}");
                    TcpListener::bind(&fallback)?
                } else {
                    return Err(MineError::from(err));
                }
            }
        };

        let local_addr = listener.local_addr().ok();
        println!("==================================================");
        println!("  Rust Minecraft 1.12.2 Server (Protocol #340)   ");
        println!("  Listening on: {:?}", local_addr);
        println!("  Connect with Minecraft Java Edition 1.12.2      ");
        println!("  Server Address: localhost:25565 or 127.0.0.1:25565");
        println!("==================================================");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let entity_id = self.entity_counter.fetch_add(1, Ordering::SeqCst);
                    println!(
                        "[{:?}] New TCP connection established (Assigned Entity ID: {})",
                        stream.peer_addr().ok(),
                        entity_id
                    );

                    thread::spawn(move || {
                        let mut conn = Connection::new(stream, entity_id);
                        if let Err(e) = conn.handle() {
                            eprintln!("Connection handler terminated with error: {e}");
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting TCP connection: {e}");
                }
            }
        }

        Ok(())
    }
}
