/* 1. Simple Client
Build a simple client that can run the two following commands:
./simple --mode=cache --times=10
./simple --mode=read
The cache mode should listen to a websocket for given number of times(seconds) only for the USD
prices of BTC. Example is given here
https://binance-docs.github.io/apidocs/websocket_api/en/#symbol-price-ticker, any other websocket is
also fine like kucoin, gemini, gateio, bybit etc. Calculate the average of these prices, say XXX. Then
print "Cache complete. The average USD price of BTC is: XXX"
Save both the result of the aggregate and the data points used to create the aggregate to a file.
The read mode should just read from the file and print the values to the terminal. */

use clap::{App, Arg};
use tokio_tungstenite::{connect_async, WebSocketStream};
use tokio::sync;
use tokio::io::{self, AsyncBufReadExt};
use futures_util::StreamExt;
use serde_derive::{Serialize,Deserialize};
use std::io::{Write,BufRead, BufReader};
use std::fs::{File, OpenOptions};



//structure of trade log
#[derive(Serialize, Deserialize)]
struct TradeLog {
    e: String,
    E: u64,
    s: String,
    t: u64,
    p: String,
    q: String,
    b: u64,
    a: u64,
    T: u64,
    m: bool,
    M: bool,
}



//to make connetion with websocket, create json and providing output
async fn getting_data(time: u64) {
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

    //collect all p values in vector
    let p_values: Vec<f64> = trade_logs.iter().map(|log| log.p.parse::<f64>().unwrap_or(0.0)).collect();
    let total_price: f64 = p_values.iter().sum();

    //calculating avg price
    let average_price = if !trade_logs.is_empty() {
        total_price / trade_logs.len() as f64
    } else {
        0.0
    };

    //serializing to json P
    let make_json_async = tokio::spawn(async move{
        let json_data = serde_json::to_string_pretty(&trade_logs).expect("Failed to serialize to JSON");
        std::fs::write("C://codezeros_workspace//practice_tasks//simple_client//btc.json", json_data).expect("Failed to write to file");
    });

    let make_output_file_async = tokio::spawn(async move {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open("C://codezeros_workspace//practice_tasks//simple_client//output.txt").expect("Failed to open or create file");

        // Write p values and average price to the file
        writeln!(file, "p values: {:?}", p_values).expect("Failed to write p values to file");
        writeln!(file, "avg P: {}", average_price).expect("Failed to write average price to file");
    });

    tokio::join!(make_json_async,make_output_file_async);

    println!("Cache complete. The average USD price of BTC is: {}", average_price);

}


fn reading_data(file_path : &str) {

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


#[tokio::main]
async fn main() {
    let matches = App::new("Birthday Greeting")
    .version("1.0")
    .author("Your Name")
    .about("reading args")
    .arg(
        Arg::with_name("mode")
            .short("mode")
            .long("mode")
            .value_name("MODE")
            .help("mode of task")
            .required(false)
            .takes_value(true),
    )
    .arg(
        Arg::with_name("times")
            .short("times")
            .long("times")
            .value_name("TIMES")
            .help("Takes time seconds")
            .required(false)
            .takes_value(true),
    )
    .get_matches();

    let mode_input = matches.value_of("mode");
    let mut times = matches.value_of("times");
    let mut time :u64 = 0; 

    if let Some(mode) = mode_input {
        //matching mode

        //for cache mode
        if mode == "cache" {
            //matching time arg
            if let Some(val) = times {
                time = val.parse::<u64>().unwrap();

                println!("mode : {}, time : {}",mode, time );
                getting_data(time).await;
            }else {
                panic!("Have to give time with cache ");
            }

        // for read mode 
        }else if mode == "read" {
            //matching time arg
            if let Some(val) = times {
                panic!("wrong arguments given with mode=read")
            }else {
                let file_path = "C://codezeros_workspace//practice_tasks//simple_client//output.txt";
                reading_data(file_path);
            }
        
        //for unknown args
        }else {
            panic!("Unknown arguments have been passed");
        }
    }else {
        //for no args
        println!("No arguments passed, so you got nothing", );
    }   
}
