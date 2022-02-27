
use std::{env, io::Error};
use futures_util::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};


fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(serve_tcp()).unwrap();
}

async fn serve_tcp() -> Result<(), Error> {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let tcp_socket = TcpListener::bind(&addr).await;
    let tcp_listener = tcp_socket.expect("Failed to bind");
    println!("Accepting TCP on: {}", addr);

    loop {
        match tcp_listener.accept().await {
            Ok((tcp_stream, _)) => { tokio::spawn(accept_connection(tcp_stream)); },
            Err(e) => println!("Failed to accept connection: {}", e),
        }
    }
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (write, read) = ws_stream.split();
    // We should not forward messages other than text or binary.
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}
