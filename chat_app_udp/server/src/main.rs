use tokio::sync::{mpsc, Mutex};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() {

    //binding the UDP socket to the server address
    let server_addr = "127.0.0.1:8080";
    let socket = tokio::net::UdpSocket::bind(server_addr).await.expect("Failed to bind UDP socket");

    println!("Server listening on {}", server_addr);

    //creating a shared state for managing connected clients using a mutex protected hashmap to avoid data race and safely sharing between channels
    let clients = Arc::new(Mutex::new(HashMap::new()));

    //creating an unbounded channel for receiving messages from all clients
    let (tx,rx) = mpsc::unbounded_channel::<(String, String)>();

    //spawning a task to handle incoming messages
    tokio::spawn(handle_messages(clients.clone(), rx));

    loop {
        //buffer to store messages
        let mut buf = vec![0; 1024];
        let (len, addr) = socket.recv_from(&mut buf).await.expect("Failed to receive data");

        let msg = String::from_utf8_lossy(&buf[..len]).into_owned();
        let sender_addr = addr.to_string();

        //trying to send the message, but handle backpressure gracefully
        if let Err(err) = tx.send((msg.clone(), sender_addr.clone())) {
            eprintln!("Failed to send message: {:#?}", err);
        }

        //retrieving the list of connected clients from the shared state
        let clients = clients.lock().await;

        //broadcasing the message to all connected clients
        for (client_addr, _) in clients.iter() {
            if client_addr != &sender_addr {
                if let Err(err) = socket.send_to(msg.as_bytes(), client_addr).await {
                    eprintln!("Failed to send message: {:?}", err);
                }
            }
        }
    }
}

//function for handling incoming messages from clients
async fn handle_messages(
    clients: Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<String>>>>,
    mut rx: mpsc::UnboundedReceiver<(String, String)>,
) {
    //matching pattern for received messages
    while let Some((msg, sender_addr)) = rx.recv().await {
        println!("Received message from {} {}", sender_addr, msg);

        //obtaining a lock on the clients hashmap
        let mut clients = clients.lock().await;

        // If the sender is a new client, add them to the list of clients
        if !clients.contains_key(&sender_addr) {
            let (tx, _) = mpsc::channel(100);
            //The key is the client's address, and the value is a Sender to communicate with that client.
            clients.insert(sender_addr.clone(), tx);
        }
    }
}
