use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() {
    
    // reading user's name
    println!("Enter your name:");
    let mut name = String::new();
    std::io::stdin().read_line(&mut name).expect("Failed to read from stdin");
    let name = name.trim();

    // used Arc to share ownership of the name between main and the spawned tasks
    let shared_name = Arc::new(name.to_owned());
    println!("Welcome to the chat!!");

    //connecting to server stream
    let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let (reader, mut writer) = tokio::io::split(stream);

    writer.write_all(format!("{} has joined the chat \n", name).as_bytes()).await.expect("failed to send joining message");
    
    //reading and writing messages concurrently by spawning 
    let reader_task = tokio::spawn({
        async move {
            read_messages(BufReader::new(reader)).await;
        }
    });

    let writer_task = tokio::spawn({
        let shared_name = Arc::clone(&shared_name);
        async move {
            write_messages(writer, &shared_name).await;
        }
    });

    //joining spawned task before main loop ends
    tokio::try_join!(reader_task, writer_task).expect("Failed to run tasks");
}


// function for reading messages from TcpStream
async fn read_messages(mut reader: BufReader<tokio::io::ReadHalf<TcpStream>>) {
    loop {
        let mut buffer = String::new();
        match reader.read_line(&mut buffer).await {
            Ok(0) => {
                // Connection closed
                break;
            }
            Ok(_) => {
                print!("{}", buffer);
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                break;
            }
        }
    }
}


// function for writing message on TcpStream
async fn write_messages(mut writer: tokio::io::WriteHalf<TcpStream>, name: &Arc<String>) {

    loop {
        // getiing user's message
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).expect("Failed to read from stdin");

        // formatting message and writing on stream
        let formatted_message = format!("{}: {}", name, message);
        writer.write_all(formatted_message.as_bytes()).await.expect("Failed to write to socket");
        
        //the flush method helps to ensure that any buffered data is sent immediately.
        writer.flush().await.expect("Failed to flush socket");
    }
}
