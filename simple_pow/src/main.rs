use sha2::{Sha256,Digest};
use chrono::prelude::*;
use hex;


//structure of the block
#[derive(Debug)]
struct Block {

    block_id : u64,
    data : String,
    prev_hash : String,
    timestamp : i64,
    nonce : u64,
    block_hash : String,
}

//associated methods for Block
impl Block {

    //for creating block
    fn create_block(block_id:u64, data: String, prev_hash : String) -> Block {
        let mut block = Block {
            block_id : block_id,
            data : data,
            prev_hash : prev_hash,
            timestamp : (Utc::now()).timestamp(),
            nonce : 0,
            block_hash : String::new(),
        };

        block.block_hash = block.calc_hash();
        block
    }

    //for calculating hash of block
    fn calc_hash(&self) -> String {

        let input_data = self.data.clone() + &(self.prev_hash) + &(self.nonce.to_string());

        let mut hasher =  Sha256::new();
        hasher.update(input_data.clone());
        let hashed_result = hasher.finalize();
        let hash = hex::encode(&hashed_result);
        hash
    }
    

    //for mining new block
    fn mine_block(&mut self, difficulty : String) {
        
        //setting difficulty
        while self.block_hash[0..difficulty.len()] != difficulty {
            self.nonce += 1;
            self.block_hash = self.calc_hash();
        }
        self.timestamp = (Utc::now()).timestamp();
        println!("Block {} mined: {}",self.block_id, self.block_hash);
    }
}

//structure of the chain
#[derive(Debug)]
struct Blockchain {
    chain : Vec<Block>,
}


//associated method of blockchain
impl Blockchain {

    //initiating chain
    fn new() -> Blockchain {
        let mut new_chain : Vec<Block> = Vec::new();
        new_chain.push(Block::create_block(0, String::from("Genesis block"),String::from("0")));

        Blockchain { chain : new_chain }
    }

    //for adding new block
    fn add_new_block(&mut self, data : String, difficulty : String) {
        let block_id = (self.chain.last().unwrap().block_id) + 1;
        let prev_hash = self.chain.last().unwrap().block_hash.clone();

        let mut block = Block::create_block(block_id, data, prev_hash);
        block.mine_block(difficulty);
        self.chain.push(block);

    }
}

fn main() {

  let mut chain_1 = Blockchain::new();

  //pass data and difficulty to add block
  chain_1.add_new_block(String::from("some set of transactions"), String::from("0000"));

  chain_1.add_new_block(String::from("some set of transactions"), String::from("00000"));

  chain_1.add_new_block(String::from("some set of transactions"), String::from("0000"));

  for block in chain_1.chain {
    println!("{:?}", block );
  }

}
