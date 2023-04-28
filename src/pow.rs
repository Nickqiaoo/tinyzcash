use num::{bigint::BigUint,ToPrimitive};
use sha2::{Sha256, Digest};
use crate::block::Block;
use hex::encode;

const TARGET_BITS: u32 = 24;
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
        let data = &self.block.data;
        let timestamp = self.block.timestamp.to_le_bytes();
        let target_bits = TARGET_BITS.to_le_bytes();
        let nonce_bytes = nonce.to_le_bytes();

        let mut bytes = vec![];
        bytes.extend_from_slice(&prev_block_hash);
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(&timestamp);
        bytes.extend_from_slice(&target_bits);
        bytes.extend_from_slice(&nonce_bytes);

        bytes
    }

    pub fn run(&self) -> (u64, [u8; 32]) {
        let mut hash_int : BigUint;
        let mut hash = [0u8; 32];
        let mut nonce = 0;

        println!("Mining the block containing \"{}\"", String::from_utf8_lossy(&self.block.data));
        while nonce < MAX_NONCE {
            let data = self.prepare_data(nonce);

            let mut hasher = Sha256::new();
            hasher.update(&data);
            let result = hasher.finalize();

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