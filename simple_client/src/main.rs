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

/*
2. Simulated distributed client
Extend the solution to Q1 by instantiating 5 client processes and one aggregator process.
a. All client processes start at the same tick of the time, say 10:01:01 AM.
b. Client process read values from the websocket for 10 seconds and computes the average and
sends it to the aggregator process.
c. Aggregator process waits for the average values from all the 5 processes. Upon getting all the
values it computes yet another average and displays on the screen.
*/

/*
3. Using signatures
Extend the solution to Q2 where the clients send the signed messages to the aggregator. And the
aggregator validates the signatures and then computes the average of averages. Any signature
scheme is fine. Set up the premise such that all the processes knows the public keys of all other
processes before hand.

used - rsa digital signature
*/

use clap::{App, Arg};
use lib::{aggregator, client_getting_data, reading_data};
use rand::rngs::OsRng;
use rsa::{RSAPrivateKey, RSAPublicKey};
use std::sync::Arc;
use tokio::sync;


#[tokio::main]
async fn main() {

    let matches = App::new("simple client")
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


    //parsing inputs
    let mode_input = matches.value_of("mode");
    let times = matches.value_of("times");
    let mut time: u64 = 0;

    //creating public and private key
    let mut rng = OsRng;
    let bits = 2048;
    let private_key = RSAPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RSAPublicKey::from(&private_key);
    let private_key = Arc::new(private_key);


    //matching mode
    if let Some(mode) = mode_input {

        //for cache mode
        if mode == "cache" {

            let (tx, rx) = sync::mpsc::channel::<(usize, [u8; 8], Vec<u8>)>(5);

            //matching time arg
            if let Some(val) = times {
                
                time = val.parse::<u64>().unwrap();
                println!("caching data from 5 clients for {} seconds", time);

                // spawning 5 client
                let handles = (1..=5)
                    .map(|id| {
                        let tx = tx.clone();
                        let pri_key = Arc::clone(&private_key);
                        println!("client {}: collecting data...", id);
                        tokio::spawn(client_getting_data(id, time, tx, pri_key))
                    })
                    .collect::<Vec<_>>();

                // spawning aggregator process
                let pub_key = Arc::new(public_key.clone());
                let aggregator_handle = tokio::spawn(aggregator(rx, pub_key));

                for handle in handles {
                    handle.await.expect("Failed to join client task");
                }

                // Drop the sender side to close the channel and signal to the aggregator
                drop(tx);

                // Wait for aggregator process to finish
                aggregator_handle
                    .await
                    .expect("Failed to join aggregator task");
            } else {
                panic!("Have to give time with cache");
            }

        // for read mode
        } else if mode == "read" {
            //matching time arg
            if let Some(val) = times {
                panic!("wrong arguments given with mode=read")
            } else {
                for x in 1..=5 {
                    let file_path = format!("output{}.txt", x);
                    println!("reading file : {}", x);
                    reading_data(&file_path);
                    println!(" ");
                }
            }

        //for unknown args
        } else {
            panic!("Unknown arguments have been passed");
        }
    } else {
        //for no args
        println!("No arguments passed, so you got nothing",);
    }
}
