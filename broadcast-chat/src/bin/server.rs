use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

/// Tangani satu koneksi client
async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Kirim pesan selamat datang ke client ini
    ws_stream
        .send(Message::text("Welcome to chat! Type a message".to_string()))
        .await?;

    // Subscribe ke broadcast channel untuk menerima pesan dari semua client
    let mut bcast_rx = bcast_tx.subscribe();

    // Gunakan loop untuk menangani:
    // - Pesan masuk dari client ini (kirim ke semua)
    // - Pesan dari broadcast (kirim ke client ini)
    loop {
        tokio::select! {
            // Terima pesan dari client ini
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        let msg = msg.as_text().unwrap_or_default();
                        println!("From client {addr:?} {msg:?}");
                        // Broadcast pesan ke semua client
                        bcast_tx.send(msg.to_string())?;
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()), // Client disconnect
                }
            }
            // Terima pesan dari broadcast (dari client lain)
            msg = bcast_rx.recv() => {
                // Kirim pesan ke client ini
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();

        // Spawn task terpisah untuk setiap client
        tokio::spawn(async move {
            // Upgrade koneksi TCP ke WebSocket
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, bcast_tx).await
        });
    }
}