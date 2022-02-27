
use std::{env, io::Error};
use std::collections::HashMap;
use futures_util::{future, StreamExt, SinkExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

type ClientID = u64;

#[derive(Default)]
pub struct DB {
    next_client_id: ClientID,
}

impl DB {
    pub fn next_client_id(&mut self) -> ClientID {
        self.next_client_id += 1;
        self.next_client_id
    }
}

type WSConnections = HashMap<ClientID, WebSocketStream<TcpStream>>;

fn main() {
    let db = DB::default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(expose_via_tcp(db)).unwrap();
}

async fn expose_via_tcp(mut db: DB) -> Result<(), Error> {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let tcp_socket = TcpListener::bind(&addr).await;
    let tcp_listener = tcp_socket.expect("Failed to bind");
    println!("Accepting TCP on: {}", addr);

    let mut clients: WSConnections = HashMap::new();

    loop {
        match tcp_listener.accept().await {
            Err(e) => println!("Failed to accept connection: {}", e),
            Ok((tcp_stream, _addr)) => {

                let ws_stream = tokio_tungstenite::accept_async(tcp_stream).await
                    .expect("Error during the websocket handshake occurred");

                // Storing client in table
                let client_id = db.next_client_id();
                clients.insert(client_id, ws_stream);

                let ws_stream = clients.get_mut(&client_id).unwrap();
                let message = format!("Your cid is {}", client_id);

                ws_stream.send(Message::Text(message)).await
                    .expect("Could not send message");

                // tokio::spawn(accept_connection(tcp_stream));
            },
        }
    }
}

async fn accept_connection(stream: TcpStream) {
    let ws_stream = tokio_tungstenite::accept_async(stream).await
        .expect("Error during the websocket handshake occurred");

    let (write, read) = ws_stream.split();

    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}
