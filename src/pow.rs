use num::{bigint::BigUint,ToPrimitive};
use sha2::{Sha256, Digest};
use crate::block::Block;
use hex::encode;

const TARGET_BITS: u32 = 8;
const MAX_NONCE: u64 = std::i64::MAX as u64;

pub struct ProofOfWork<'a> {
    block: &'a Block,
    target: BigUint,
}

impl<'a> ProofOfWork<'a> {
    pub fn new(block: &'a Block) -> Self {
        let target = BigUint::from(1u64)
                    << (256 - TARGET_BITS).to_usize().unwrap();

        ProofOfWork { block, target }
    }

    fn prepare_data(&self, nonce: u64) -> Vec<u8> {
        let prev_block_hash = self.block.prev_block_hash.clone();
        let mut trans = self.block.hash_transactions();
        let timestamp = self.block.timestamp.to_le_bytes();
        let target_bits = TARGET_BITS.to_le_bytes();
        let nonce_bytes = nonce.to_le_bytes();

        let mut bytes = vec![];
        bytes.extend_from_slice(&prev_block_hash);
        bytes.append(&mut trans);
        bytes.extend_from_slice(&timestamp);
        bytes.extend_from_slice(&target_bits);
        bytes.extend_from_slice(&nonce_bytes);

        bytes
    }

    
    pub fn validate(&self) -> bool {
        let data = self.prepare_data(self.block.nonce);
        let hash = Sha256::digest(&data);
        let hash_int = BigUint::from_bytes_le(&hash);

        hash_int < self.target
    }

    pub fn run(&self) -> (u64, [u8; 32]) {
        let mut hash_int : BigUint;
        let mut hash = [0u8; 32];
        let mut nonce = 0;

        println!("Mining the block");
        while nonce < MAX_NONCE {
            let data = self.prepare_data(nonce);

            let result = Sha256::digest(&data);
            hash.copy_from_slice(&result.as_slice());
            hash_int = BigUint::from_bytes_le(&hash);
            hash.reverse();

            if hash_int < self.target {
                print!("\r{}", encode(hash));
                break;
            } else {
                nonce += 1;
            }
        }
        println!("\n");

        (nonce, hash)
    }
}