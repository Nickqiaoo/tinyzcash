use crate::block::Block;

pub struct Blockchain {
    pub blocks : Vec<Block>
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain { blocks: vec![Block::genesis()] }
    }

    pub fn add_block(&mut self, data: &str) {
        let prev_block_hash = self.blocks.last().unwrap().hash.clone();
        let block = Block::new(data, &prev_block_hash);
        self.blocks.push(block);
    }
}