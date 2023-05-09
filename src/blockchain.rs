use crate::block::Block;
use sled;

pub struct Blockchain {
    pub tip: Vec<u8>,
    pub db: sled::Db,
}

impl Blockchain {
    pub fn new() -> Self {
        let db = match sled::open(db_file) {
            Ok(db) => db,
            Err(e) => panic!("Failed to open database: {}", e),
        };
        let mut tip = vec![];

        db.transaction(|tx| {
            let b = db.open_tree(blocks_bucket).unwrap();
            if b.is_empty() {
                let genesis = Block::genesis();
                b.insert(&genesis.hash, genesis.serialize()).unwrap();
                b.insert(b"l", genesis.hash).unwrap();
                tip = genesis.hash.to_vec();
            } else {
                tip = b.get(b"l").unwrap().unwrap().to_vec();
            }
            Ok(())
        })
        .unwrap();

        Blockchain { tip, db }
    }

    pub fn add_block(&mut self, data: &str) {
        let last_hash: Vec<u8>;

        {
            let b = self.db.open_tree("blocksBucket").unwrap();
            last_hash = b.get(b"l").unwrap().unwrap().to_vec();
        }

        let new_block = Block::new(data, &last_hash);

        {
            let mut b = self.db.open_tree("blocksBucket").unwrap();
            b.insert(new_block.hash.clone(), new_block.serialize())
                .unwrap();
            b.insert(b"l", new_block.hash.clone()).unwrap();
            self.tip = new_block.hash;
        }
    }
}
