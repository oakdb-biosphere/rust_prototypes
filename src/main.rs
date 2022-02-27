
use std::{env, io::Error};
use std::collections::HashMap;
use std::sync::{Arc};
use futures::lock::Mutex;
use futures_util::{SinkExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{WebSocketStream};
use tokio_tungstenite::tungstenite::Message;


/// A unique value that will be assigned to each
/// database client
type ClientID = u64;

/// A primitive database structure whose operations
/// will be exposed via WebSocket server
#[derive(Default)]
pub struct Database {
    next_client_id: ClientID,
}
impl Database {
    pub fn next_client_id(&mut self) -> ClientID {
        self.next_client_id += 1;
        self.next_client_id
    }
}

/// A table mapping database clients to websocket connections
type WSClientTable = HashMap<ClientID, WebSocketStream<TcpStream>>;

/// Aliases for types concerned with shared-mutability
type SharedWSClientTable = Arc<Mutex<WSClientTable>>;
type SharedDatabase = Arc<Mutex<Database>>;


fn main() {
    let db = Database::default();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(expose_via_tcp(db)).unwrap();
}


/// Continuously accept TCP connections, and try to interpret
/// incoming messages as database operations
async fn expose_via_tcp(db: Database) -> Result<(), Error> {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let tcp_socket = TcpListener::bind(&addr).await;
    let tcp_listener = tcp_socket.expect("Failed to bind");
    println!("Accepting TCP on: {}", addr);

    // Making shareable global objects
    let clients: WSClientTable = HashMap::new();
    let clients = Arc::new(Mutex::new(clients));
    let db = Arc::new(Mutex::new(db));

    loop {
        match tcp_listener.accept().await {
            Err(e) => println!("Failed to accept connection: {}", e),
            Ok((tcp_stream, addr)) => {

                // Creating new references to shared objects
                let shared_clients = Arc::clone(&clients);
                let shared_db = Arc::clone(&db);

                println!("Receiving new connection: {:?}", addr);
                tokio::spawn(handle_connection(tcp_stream, shared_clients, shared_db));
            },
        }
    }
}

/// Performs the WebSocket protocol handshake, and handles
/// the sending and receiving of messages
async fn handle_connection(
    tcp_stream: TcpStream,
    shared_clients: SharedWSClientTable,
    shared_db: SharedDatabase
) {
    let ws_stream = tokio_tungstenite::accept_async(tcp_stream).await
        .expect("Error during the websocket handshake occurred");

    // Getting mutable references to shared objects
    let mut clients = shared_clients.lock().await;
    let mut db = shared_db.lock().await;

    // Adding the new client to the collection
    let client_id = db.next_client_id();
    clients.insert(client_id, ws_stream);

    // Preparing a message to send to the client
    let ws_stream = clients.get_mut(&client_id).unwrap();
    let message = format!("Your cid is {}", client_id);

    ws_stream.send(Message::Text(message)).await
        .expect("Failed to send message")
}
