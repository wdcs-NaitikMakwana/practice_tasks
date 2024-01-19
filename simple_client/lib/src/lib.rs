use tokio::sync;
use tokio::io;
use tokio_tungstenite::{connect_async, WebSocketStream};
use futures_util::StreamExt;
use serde_derive::{Serialize,Deserialize};
use std::io::{Write,BufRead, BufReader};
use std::fs::{File, OpenOptions};
use std::sync::Arc;
use crypto::{sha2::Sha256, digest::Digest};
use rand::rngs::OsRng;
use rsa::{PublicKey, RSAPrivateKey, RSAPublicKey,PaddingScheme};

//structure of trade log
#[derive(Serialize, Deserialize)]
pub struct TradeLog {
    pub e: String,
    pub E: u64,
    pub s: String,
    pub t: u64,
    pub p: String,
    pub q: String,
    pub b: u64,
    pub a: u64,
    pub T: u64,
    pub m: bool,
    pub M: bool,
}




//reading data from file
pub fn reading_data(file_path : &str) {

    let file = File::open(file_path).expect("Failed to open file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(line_content) = line {
            println!("{}", line_content);
        } else {
            eprintln!("Error reading line from file");
        }
    }

}




//client function
pub async fn client_getting_data(id:usize , time: u64, sender:sync::mpsc::Sender<(usize,[u8; 8], Vec<u8>)>, private_key: Arc<RSAPrivateKey>) 
{

    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";
    let (mut ws_stream, _) = connect_async(url).await.expect("failed to connect");

    let (_, mut read) = ws_stream.split();

    //vec to store data
    let mut received_data = Vec::new();

    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_secs() < time {
        match read.next().await {
            Some(Ok(msg)) => {
                if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                    //println!("Received a message: {}", text);
                    received_data.push(text);
                }
            }
            Some(Err(e)) => eprintln!("Error receiving message: {}", e),
            None => {
                println!("Connection closed");
                break;
            }
        }
    }

    // Parse and convert received JSON strings into TradeLog structs
    let trade_logs: Vec<TradeLog> = received_data.iter()
        .filter_map(|msg| serde_json::from_str(msg).ok())
        .collect();

    //collect all prices values in vector
    let p_values: Vec<f64> = trade_logs.iter().map(|log| log.p.parse::<f64>().unwrap_or(0.0)).collect();
    let total_price: f64 = p_values.iter().sum();

    //calculating avg price
    let average_price = if !trade_logs.is_empty() {
        total_price / trade_logs.len() as f64
    } else {
        0.0
    };

    //serializing to json 
    let make_json_async = tokio::spawn(async move{
        let json_data = serde_json::to_string_pretty(&trade_logs).expect("Failed to serialize to JSON");
        std::fs::write((format!("btc{}.json",id)).as_str(), json_data).expect("Failed to write to file");
    });

    let make_output_file_async = tokio::spawn(async move {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open((format!("output{}.txt",id)).as_str()).expect("Failed to open or create file");

        // Write p values and average price to the file
        writeln!(file, "p values: {:?}", p_values).expect("Failed to write p values to file");
        writeln!(file, "avg P: {}", average_price).expect("Failed to write average price to file");
    });

    tokio::join!(make_json_async,make_output_file_async);

    println!("Cache complete. The average USD price of BTC calculated by client {} is: {}", id, average_price);

    //converting into array of bytes for authentication
   let avg_price_bytes = average_price.to_le_bytes();

    //signing the data with sender's private key
   let signature = private_key.sign(PaddingScheme::PKCS1v15Sign {hash: None }, &avg_price_bytes).expect("Failed to sign data");

    //Sending data as well as signature as a tuple
   let data = (id,avg_price_bytes, signature);

    sender.send(data).await.expect("failed to send avg price");

}






//aggregator function

pub async fn aggregator(mut receiver: sync::mpsc::Receiver<(usize,[u8; 8], Vec<u8>)>, public_key: Arc<RSAPublicKey>) {

    let mut rec_avg_prices = Vec::new();
    let pub_key = Arc::clone(&public_key);

    for _  in 0..5 {
        if let Some((id,avg_price_bytes,signature)) = receiver.recv().await {

            //verifying signature with senders public key
            if pub_key.verify(PaddingScheme::PKCS1v15Sign { hash: None }, &avg_price_bytes, &signature)
            .is_ok() {
                println!("client {} : Authentication successful!!",id);
                let avg_price = f64::from_le_bytes(avg_price_bytes);
                rec_avg_prices.push(avg_price);
            }else {
                panic!("Authentication of client {} fail. keys do not match", id);
            }

        }
    }

    //calculating aggregated avg
    let total_avg: f64 = rec_avg_prices.iter().sum();
    let final_avg = if !rec_avg_prices.is_empty() {
        total_avg / rec_avg_prices.len() as f64
    } else {
        0.0
    };

    println!("Aggregator: received avg from 5 client and the final average USD price of BTC is: {}", final_avg);

}