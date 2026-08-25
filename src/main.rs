use mine::server::Server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "25565".to_string());
    // Use [::]:port to support both IPv4 (127.0.0.1) and IPv6 (::1 / localhost) (something related to dual-stack sockets, idk tbh)
    let bind_addr = format!("[::]:{port}");

    let server = Server::new(bind_addr);
    server.run()?;

    Ok(())
}
