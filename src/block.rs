use std::{time, vec};
use crate::{pow::ProofOfWork, transaction::Transaction};
use serde::{Serialize, Deserialize};
use serde_json;
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub prev_block_hash: Vec<u8>,
    pub transactions: Vec<Transaction>,
    pub timestamp: i64,
    pub hash: Vec<u8>,
    pub nonce: u64,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, prev_block_hash: Vec<u8>) -> Self {
        let timestamp = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let mut block = Block {
            prev_block_hash: prev_block_hash,
            transactions,
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

    pub fn genesis(coinbase: Transaction) -> Self {
        Block::new(vec![coinbase], Vec::new())
    }

    pub fn serialize(&self) -> Vec<u8> {
        let json_str = serde_json::to_string(self).unwrap();
        return json_str.into_bytes();
    }

    pub fn hash_transactions(&self) -> Vec<u8> {
        let mut tx_hashes = Vec::new();

        for tx in &self.transactions {
            tx_hashes.push(tx.id.clone());
        }

        let concatenated_hashes = tx_hashes.concat();
        Sha256::digest(&concatenated_hashes).to_vec()
    }
}

pub fn deserialize_block(d: &[u8]) -> Result<Block, serde_json::Error> {
    let block: Block = serde_json::from_slice(d)?;
    Ok(block)
}