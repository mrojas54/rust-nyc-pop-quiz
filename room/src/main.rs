//! Binds a socket and serves the room.
//!
//! Scaffold. The address is fixed here on purpose: T-09 owns deploy and will
//! decide how the bind address is configured on Fly. Nothing should grow a
//! configuration story for it before then.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("room listening on http://{addr}");
    axum::serve(listener, room::router()).await?;
    Ok(())
}
