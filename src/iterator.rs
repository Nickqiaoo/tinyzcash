use sled;
use crate::block::{Block, deserialize_block};

pub struct BlockchainIterator<'a> {
    pub current_hash: Vec<u8>,
    pub db: &'a sled::Db,
}

impl<'a> BlockchainIterator<'a> {
    pub fn next(&mut self) -> Option<Block> {
        let mut block = None;
        let encoded_block;
        let b = self.db.open_tree("blocksBucket").unwrap();
        match  b.get(&self.current_hash) {
            Ok(res) => {
                match res {
                    Some(hash) => encoded_block = hash,
                    None => return block,
                }
            },
            Err(_) => return block,
        }

        block = Some(deserialize_block(&encoded_block).unwrap());
        if let Some(ref b) = block {
            self.current_hash = b.prev_block_hash.clone();
        }
        block
    }
}