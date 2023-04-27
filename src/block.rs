mod block {
use std::convert::TryInto;
use sha2::{Sha256, Digest};
struct Block {
    prev_block_hash: Vec<u8>,
    data: Vec<u8>,
    timestamp: i64,
    hash: Vec<u8>,
}

impl Block {
    fn new(data: &str, prev_block_hash: &[u8]) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let block = Block {
            prev_block_hash: prev_block_hash.to_vec(),
            data: data.as_bytes().to_vec(),
            timestamp,
            hash: vec![],
        };
        
        block.set_hash();
        block_clone
    }

    fn SetHash(&mut self){
        let timestamp = self.timestamp.to_string().into_bytes();
        let headers = [self.prev_block_hash.as_slice(), self.data.as_slice(), &timestamp[..]].concat();
        let mut hasher = Sha256::new();
        hasher.update(headers);
        let hash = hasher.finalize();
        self.hash = hash.to_vec();
    }
}
}