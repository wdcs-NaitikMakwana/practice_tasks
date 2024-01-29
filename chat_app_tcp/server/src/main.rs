use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {

    //binding to address
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server listening on 127.0.0.1:8080");

    let (tx, _rx) = broadcast::channel(100);

    while let Ok((socket, _)) = listener.accept().await {
        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(async move {
            handle_client(socket, tx, rx).await;
        });
    }
}

//function for handling client process
async fn handle_client(mut socket: TcpStream, tx: broadcast::Sender<String>, mut rx: broadcast::Receiver<String>) 
{   
    let (mut reader, mut writer) = socket.split();
    let mut buffer = [0; 1024];
    
    loop {
        tokio::select! {
            Ok(bytes_read) = reader.read(&mut buffer) => {
                if bytes_read == 0 {
                    // connection is closed
                    break;
                } else {
                    // broadcasting the received message to all clients
                    let message = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                    println!("chat log : {}",message.clone() );
                    tx.send(message).unwrap();
                }
            }
            Ok(message) = rx.recv() => {
                // writing the message back to the client
                if let Err(e) = writer.write_all(message.as_bytes()).await {
                    eprintln!("Error writing to socket: {}", e);
                    break;
                }
            }
        }
    }
}