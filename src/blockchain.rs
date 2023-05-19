use std::collections::HashMap;
use crate::{block::Block, iterator::BlockchainIterator, transaction::{Transaction, new_coinbase_tx}, transaction_output::TXOutput};
use sled;

const COINBASEDATA: &str = "coinbase";

pub struct Blockchain {
    pub tip: Vec<u8>,
    pub db: sled::Db,
}

impl Blockchain {
    pub fn new(address: &str) -> Self {
        let db = match sled::open("db.file") {
            Ok(db) => db,
            Err(e) => panic!("Failed to open database: {}", e),
        };
        let tip: Vec<u8>;
        let b = db.open_tree("blocksBucket").unwrap();

        if b.is_empty() {
            let genesis = Block::genesis(new_coinbase_tx(address, COINBASEDATA));
            b.insert(&genesis.hash, genesis.serialize()).unwrap();
            b.insert(b"l", genesis.hash.as_slice()).unwrap();
            tip = genesis.hash.to_vec();
            b.flush().unwrap();
        } else {
            tip = b.get(b"l").unwrap().unwrap().to_vec();
        }

        Blockchain { tip, db }
    }

    pub fn find_utxo(&self, address: &str) -> Vec<TXOutput> {
        let mut utxos = Vec::new();
        let unspent_transactions = self.find_unspent_transactions(address);

        for tx in unspent_transactions {
            for out in tx.vout {
                if out.can_be_unlocked_with(address) {
                    utxos.push(out.clone());  // Assuming TXOutput implements Clone trait
                }
            }
        }

        utxos
    }

    pub fn mine_block(&mut self, transactions :Vec<Transaction>) {
        let b = self.db.open_tree("blocksBucket").unwrap();
        let prev_block_hash = b.get(b"l").unwrap().unwrap().to_vec();
        
        let new_block = Block::new(transactions, prev_block_hash);
        self.tip = new_block.hash.to_vec();

        b.insert(&new_block.hash, new_block.serialize()).unwrap();
        b.insert(b"l", new_block.hash.as_slice()).unwrap();
        b.flush().unwrap();
    }
    
    pub fn iterator(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip.clone(),
            db: &self.db,
        }
    }

    pub fn find_spendable_outputs(&self, address: &str, amount: i64) -> (i64, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs = HashMap::new();
        let unspent_txs = self.find_unspent_transactions(address);
        let mut accumulated = 0;

        'Work: for tx in &unspent_txs {
            let tx_id = hex::encode(tx.id.clone());

            for (out_idx, out) in tx.vout.iter().enumerate() {
                if out.can_be_unlocked_with(address) && accumulated < amount {
                    accumulated += out.value;
                    let outputs = unspent_outputs.entry(tx_id.clone()).or_insert(Vec::new());
                    outputs.push(out_idx as i32);

                    if accumulated >= amount {
                        break 'Work;
                    }
                }
            }
        }

        (accumulated, unspent_outputs)
    }
    
    fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {
        let mut unspent_txs :Vec<Transaction>= Vec::new();
        let mut spent_txos = HashMap::new();
        let mut bci = self.iterator();

        while let Some(block) = bci.next() {
            for tx in &block.transactions {
                let tx_id = hex::encode(tx.id.clone());

                // 检查所有输出是否被花费
                for (out_idx, out) in tx.vout.iter().enumerate() {
                    if spent_txos.get(&tx_id).map_or(false, |v:&Vec<i32>| v.contains(&(out_idx as i32))) {
                        continue;
                    }
                    if out.can_be_unlocked_with(address) {
                        unspent_txs.push(tx.clone());
                    }
                }

                // 更新已花费的输出
                if !tx.is_coinbase() {
                    for input in &tx.vin {
                        if input.can_unlock_output_with(address) {
                            let in_tx_id = hex::encode(input.txid.clone());
                            spent_txos.entry(in_tx_id).or_insert_with(Vec::new).push(input.vout);
                        }
                    }
                }
            }
            if block.prev_block_hash.is_empty() {
                break;
            }
        }

        unspent_txs
    }
}
