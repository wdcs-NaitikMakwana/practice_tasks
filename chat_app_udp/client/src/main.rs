use tokio::net::UdpSocket;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {

    // Binding the UDP socket for the client
    let server_addr = "127.0.0.1:8080";
    let client_addr = "127.0.0.1:0"; // Let the system assign a random port for the client

    let socket = UdpSocket::bind(client_addr).await.expect("Failed to bind UDP socket");
    // Binding another UDP socket for the server communicatio
    let server_socket = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind UDP socket");

    //user input for name
    println!("Enter your name");
    let mut name = String::new();
    std::io::stdin().read_line(&mut name).expect("failed reading name");
    let name = name.trim().to_string();

    println!("Welcome to the chat!!");
    let username = name.clone();

    //sending a joining message to the server
    socket.send_to(format!("{} has joined the chat" , username).as_bytes(), server_addr).await.expect("Failed to send join message");

    //creating a channel to handle incoming messages
    let (tx, mut rx) = mpsc::channel::<String>(100);

    //spawning a task to handle incoming messages
    tokio::spawn(handle_messages(socket, tx));

    //spawning a task to send messages to the server
    tokio::spawn(send_messages(name,server_socket, server_addr));

    //start receiving and printing messages
    while let Some(msg) = rx.recv().await {
        println!("{}", msg);
    }
}

//function for handling incoming messages
async fn handle_messages(socket: UdpSocket, tx: mpsc::Sender<String>) {
    let mut buf = vec![0; 1024];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await.expect("Failed to receive data");
        let msg = String::from_utf8_lossy(&buf[..len]).into_owned();

        //sending the message to the main loop for printing
        tx.send(msg).await.expect("Failed to send message to main loop");
    }
}

//function for sending message
async fn send_messages(name:String , socket: UdpSocket, server_addr: &str) {
    let mut input = String::new();

    loop {
        // Read user input
        std::io::stdin().read_line(&mut input).expect("Failed to read user input");
        let msg = format!("{} : {}", name, input);

        // Send the input to the server
        socket.send_to(msg.trim().as_bytes(), server_addr).await.expect("Failed to send message to server");

        // Clear the input buffer
        input.clear();
    }
}
