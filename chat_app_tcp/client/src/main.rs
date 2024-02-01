use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt,AsyncReadExt};

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

    writer.write_all(format!("{} has joined the chat \n<END>", name).as_bytes()).await.expect("failed to send joining message");
    
    //reading and writing messages concurrently by spawning 
    let reader_task = tokio::spawn({
        async move {
            read_messages(reader).await;
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

async fn read_messages(mut reader: tokio::io::ReadHalf<TcpStream>) {

    let delimiter = "<END>"; //defining delimeter
    let mut buffer = Vec::new();
    loop {
        let mut chunk = vec![0; 1024];
        match reader.read(&mut chunk).await {
            Ok(0) => {
                // Connection closed
                break;
            }
            Ok(bytes_read) => {
                //extending the buffer with the bytes that were just read from the TCP
                buffer.extend_from_slice(&chunk[..bytes_read]); 

                //The buffer.windows(delimiter.len()) part is creating an iterator over contiguous windows in the buffer that have same length as the delimiter.

                // The position method return position of the first window that equals the delimiter.
                if let Some(i) = buffer.windows(delimiter.len()).position(|window| window == delimiter.as_bytes()) {

                    let message = buffer.drain(..i).collect::<Vec<_>>(); //collecting msg before delimeter
                    buffer.drain(..delimiter.len()); // remove the delimiter from buffer
                    print!("{}", String::from_utf8_lossy(&message));
                }
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

    let delimiter = "<END>"; // defining delimeter

    loop {
        // getting user's message
        let mut message = String::new();
        std::io::stdin().read_line(&mut message).expect("Failed to read from stdin");

        // formatting message and writing on stream
        let formatted_message = format!("{}: {}", name, message);
        let mut bytes = formatted_message.as_bytes().to_vec();

        // add the delimiter to the end of the message
        bytes.extend_from_slice(delimiter.as_bytes());

        // write the message in chunks
        while !bytes.is_empty() {
            let to_write = if bytes.len() > 1024 { 1024 } else { bytes.len() };
            let chunk = bytes.drain(..to_write).collect::<Vec<_>>();
            writer.write_all(&chunk).await.expect("Failed to write to socket");
        }

        // the flush method helps to ensure that any buffered data is sent immediately.
        writer.flush().await.expect("Failed to flush socket");
    }
}