use std::time;
use sha2::{Sha256, Digest};
use crate::pow::ProofOfWork;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub prev_block_hash: Vec<u8>,
    pub data: Vec<u8>,
    pub timestamp: i64,
    pub hash: Vec<u8>,
    pub nonce: u64,
}

impl Block {
    pub fn new(data: &str, prev_block_hash: &[u8]) -> Self {
        let timestamp = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut block = Block {
            prev_block_hash: prev_block_hash.to_vec(),
            data: data.as_bytes().to_vec(),
            timestamp,
            hash: vec![],
            nonce:0,
        };
        
        let pow = ProofOfWork::new(&block);
        let (nonce, hash) = pow.run();
    
        block.hash = hash.to_vec();
        block.nonce = nonce;

        block
    }

    pub fn set_hash(&mut self){
        let timestamp = self.timestamp.to_string().into_bytes();
        let headers = [self.prev_block_hash.as_slice(), self.data.as_slice(), &timestamp[..]].concat();
        let mut hasher = Sha256::new();
        hasher.update(headers);
        let hash = hasher.finalize();
        self.hash = hash.to_vec();
    }
    pub fn genesis() -> Self {
        Block::new("Genesis Block", &[])
    }

    pub fn serialize(&self) -> Vec<u8> {
        let json_str = serde_json::to_string(self).unwrap();
        return json_str.into_bytes();
    }
}

fn deserialize_block(d: &[u8]) -> Result<Block, serde_json::Error> {
    let block: Block = serde_json::from_slice(d)?;
    Ok(block)
}