use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::io::{stdin, BufRead};
use tokio::sync::mpsc;
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Buat direktori src/bin jika belum ada
    let (mut ws_stream, _) =
        ClientBuilder::from_uri(http::Uri::from_static("ws://127.0.0.1:2000"))
            .connect()
            .await?;

    let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel();

    // Spawn thread untuk membaca input dari keyboard
    tokio::task::spawn_blocking(move || {
        let stdin = stdin();
        for line in stdin.lock().lines() {
            if let Ok(line) = line {
                stdin_tx.send(line).unwrap();
            }
        }
    });

    loop {
        tokio::select! {
            // Kirim pesan yang diketik user ke server
            line = stdin_rx.recv() => {
                match line {
                    Some(msg) => ws_stream.send(Message::text(msg)).await?,
                    None => return Ok(()), // stdin ditutup
                }
            }
            // Terima pesan dari server dan tampilkan
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => println!("From server: {}", msg.as_text().unwrap_or_default()),
                    Some(Err(e)) => return Err(e.into()),
                    None => return Ok(()), // Server disconnect
                }
            }
        }
    }
}