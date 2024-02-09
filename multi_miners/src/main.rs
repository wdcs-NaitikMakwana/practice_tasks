//multiple miners using mpsc channels and tasks

use sha2::{Sha256, Digest};
use chrono::prelude::*;
use hex;
use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::io::Write;

//structure of the block
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Block {
    block_id: u64,
    data: String,
    prev_hash: String,
    timestamp: i64,
    nonce: u64,
    block_hash: String,
}

//associate methods for Block structure
impl Block {
    // For creating block
    fn create_block(block_id: u64, data: String, prev_hash: String) -> Block {
        let mut block = Block {
            block_id: block_id,
            data: data,
            prev_hash: prev_hash,
            timestamp: Utc::now().timestamp(),
            nonce: 0,
            block_hash: String::new(),
        };
        // Calculating block hash
        block.block_hash = block.calc_hash();
        block
    }

    //function for calculating hash of block
    fn calc_hash(&self) -> String {
        // Collecting data to be hashed
        let input_data = self.data.clone() + &(self.prev_hash) + &(self.nonce.to_string());
        let mut hasher = Sha256::new();
        hasher.update(input_data.clone());
        let hashed_result = hasher.finalize();
        // Converting hashed bytes to hex form
        let hash = hex::encode(&hashed_result);
        hash
    }

    // function for mining new block
    fn mine_block(&mut self, difficulty: String) {
        //setting difficulty
        while self.block_hash[0..difficulty.len()] != difficulty {
            self.nonce += 1;
            self.block_hash = self.calc_hash();
        }
        self.timestamp = Utc::now().timestamp();
        println!("Block {} mined: {}", self.block_id, self.block_hash);
    }

    // Function to write block to file
    fn write_to_json_file(&self, filename: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)?;
        //serializing to store in json
        let serialized = serde_json::to_string(&self)?;
        writeln!(file, "{}", serialized)?;
        Ok(())
    }
}

//structure of the blockchain
#[derive(Debug)]
struct Blockchain {
    chain: Vec<Block>,
}

//associate method of blockchain
impl Blockchain {
    //initiating new blockchain and adding genesis block to the chain
    fn new() -> Blockchain {
        let mut new_chain: Vec<Block> = Vec::new();
        new_chain.push(Block::create_block(0, String::from("Genesis block"), String::from("0")));
        let new_blockchain = Blockchain { chain: new_chain };
        new_blockchain
    }

    // function for adding new block
    fn add_new_block(&mut self, data: String, difficulty: String) -> Block {
        let block_id = (self.chain.last().unwrap().block_id) + 1;
        let prev_hash = self.chain.last().unwrap().block_hash.clone();
        //creating block
        let mut block = Block::create_block(block_id, data, prev_hash);
        //mining block
        block.mine_block(difficulty);
        // Adding to blockchain
        self.chain.push(block.clone());
        block
    }

    // function to verify and add block to chain and json
    fn verify_and_add_block(&mut self, block: Block, filepath: &str) {
        //validating block
        if block.block_hash == block.calc_hash() {
            //adding to chain
            self.chain.push(block.clone());
            //writing to json
            if let Err(e) = block.write_to_json_file(filepath) {
                eprintln!("Error writing block to file: {}", e);
            }
            println!("Block verified and added: {:?}", block);
        }else {
            panic!("block is not legitimate!!")
        }
    }
}


#[tokio::main]
async fn main() {
    //creating channels for communication between nodes
    let (sender1, receiver1) = mpsc::channel::<Block>();
    let (sender2, receiver2) = mpsc::channel::<Block>();

    // initiating chain for each task
    let chain_1 = Arc::new(Mutex::new(Blockchain::new()));
    let chain_2 = Arc::new(Mutex::new(Blockchain::new()));

    //using Arc to share chain safely between channels
    let chain_1_clone = Arc::clone(&chain_1);
    let chain_2_clone = Arc::clone(&chain_2);

    //cloning receiving channel to work on spawned tasks
    let receiver1_clone = Arc::new(Mutex::new(receiver1));
    let receiver2_clone = Arc::new(Mutex::new(receiver2));

    // counter to track the round-robin behavior
    let mut counter = 1;

    // Task 1
    let sender1_clone = sender1.clone();
    let receiver2_clone_task1 = Arc::clone(&receiver2_clone);
    tokio::spawn(async move {
        loop {
            if counter == 1 {
                println!("miner 1 started mining");
                let mut chain = chain_1_clone.lock().unwrap();

                //mining new block to add it on chain
                let mined_block = chain.add_new_block(String::from("some transactions : mined by 1"), String::from("00000"));

                //writing it to local chain
                let _ = mined_block.write_to_json_file("chain_local_1.json");

                //sending it to other miner for verification so that they can add it to their chain
                sender1_clone.send(mined_block.clone()).unwrap();
            }else {
                //receiving block
                let block = receiver2_clone_task1.lock().unwrap().recv().unwrap();
                let mut chain = chain_1_clone.lock().unwrap();

                //verifying and adding it to local chain
                chain.verify_and_add_block(block.clone(), "chain_local_1.json");
            }
            //setting counter for another miner's turn
            counter = (counter % 2) + 1;
        }
    });

    // Task 2
    let sender2_clone = sender2.clone();
    let receiver1_clone_task2 = Arc::clone(&receiver1_clone);
    tokio::spawn(async move {
        loop {

            if counter == 2 {
                println!("miner 2 started mining");
                let mut chain = chain_2_clone.lock().unwrap();

                //mining new block to add it on chain
                let mined_block = chain.add_new_block(String::from("some transactions : mined by 2"), String::from("00000"));

                //writing it to local chain
                let _ = mined_block.write_to_json_file("chain_local_2.json");

                //sending it to other miner for verification so that they can add it to their chain
                sender2_clone.send(mined_block.clone()).unwrap();
            }else {
                //receiving block
                let block = receiver1_clone_task2.lock().unwrap().recv().unwrap();
                let mut chain = chain_2_clone.lock().unwrap();

                //verifying and adding it to local chain
                chain.verify_and_add_block(block.clone(), "chain_local_2.json");
            }
            //setting counter for another miner's turn
            counter = (counter % 2) + 1;
        }
    });


    //preventing main task from completing so that mining will continue in infinite loop
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait for 1 second
    }
}