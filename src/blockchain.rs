use crate::{
    block::Block,
    iterator::BlockchainIterator,
    transaction::{new_coinbase_tx, Transaction},
    transaction_output::TXOutput,
};
use std::{collections::HashMap, error::Error};

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
            let genesis = Block::genesis(new_coinbase_tx(address, COINBASEDATA, 10));
            b.insert(&genesis.hash, genesis.serialize()).unwrap();
            b.insert(b"l", genesis.hash.as_slice()).unwrap();
            tip = genesis.hash.to_vec();
            b.flush().unwrap();
        } else {
            tip = b.get(b"l").unwrap().unwrap().to_vec();
        }

        Blockchain { tip, db }
    }

    pub fn find_utxo(&self, pub_key_hash: &Vec<u8>) -> Vec<TXOutput> {
        let mut utxos = Vec::new();
        let unspent_transactions = self.find_unspent_transactions(pub_key_hash);

        for tx in unspent_transactions {
            for out in tx.vout {
                if out.is_locked_with_key(pub_key_hash) {
                    utxos.push(out.clone()); // Assuming TXOutput implements Clone trait
                }
            }
        }

        utxos
    }

    pub fn mine_block(&mut self, transactions: Vec<Transaction>) {
        let b = self.db.open_tree("blocksBucket").unwrap();
        let prev_block_hash = b.get(b"l").unwrap().unwrap().to_vec();

        for tx in &transactions {
            if !self.verify_transaction(tx) {
                panic!("Invalid transaction");
            }
        }

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

    pub fn find_spendable_outputs(
        &self,
        address: &Vec<u8>,
        amount: i64,
    ) -> (i64, HashMap<String, Vec<i32>>) {
        let mut unspent_outputs = HashMap::new();
        let unspent_txs = self.find_unspent_transactions(address);
        let mut accumulated = 0;

        'Work: for tx in &unspent_txs {
            let tx_id = hex::encode(tx.id.clone());

            for (out_idx, out) in tx.vout.iter().enumerate() {
                if out.is_locked_with_key(address) && accumulated < amount {
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

    fn find_unspent_transactions(&self, address: &Vec<u8>) -> Vec<Transaction> {
        let mut unspent_txs: Vec<Transaction> = Vec::new();
        let mut spent_txos = HashMap::new();
        let mut bci = self.iterator();

        while let Some(block) = bci.next() {
            for tx in &block.transactions {
                let tx_id = hex::encode(tx.id.clone());

                // 检查所有输出是否被花费
                for (out_idx, out) in tx.vout.iter().enumerate() {
                    if spent_txos
                        .get(&tx_id)
                        .map_or(false, |v: &Vec<i32>| v.contains(&(out_idx as i32)))
                    {
                        continue;
                    }
                    if out.is_locked_with_key(address) {
                        unspent_txs.push(tx.clone());
                    }
                }

                // 更新已花费的输出
                if !tx.is_coinbase() {
                    for input in &tx.vin {
                        if input.use_key(address) {
                            let in_tx_id = hex::encode(input.txid.clone());
                            spent_txos
                                .entry(in_tx_id)
                                .or_insert_with(Vec::new)
                                .push(input.vout);
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

    fn find_transaction(&self, id: &Vec<u8>) -> Result<Transaction, Box<dyn Error>> {
        let mut bci = self.iterator();

        loop {
            if let Some(block) = bci.next() {
                for tx in &block.transactions {
                    if tx.id == *id {
                        return Ok(tx.clone());
                    }
                }

                if block.prev_block_hash.is_empty() {
                    break;
                }
            }
        }

        Err("TransactionNotFound".into())
    }

    pub fn sign_transaction(&self, tx: &mut Transaction, priv_key: String) {
        let priv_key =
            secp256k1::SecretKey::from_slice(hex::decode(priv_key).unwrap().as_slice()).unwrap();
        let mut prev_txs = HashMap::new();

        for vin in tx.vin.iter() {
            let prev_tx_result = self.find_transaction(&vin.txid);
            if let Ok(prev_tx) = prev_tx_result {
                prev_txs.insert(hex::encode(&prev_tx.id), prev_tx);
            }
        }

        tx.sign(priv_key, &prev_txs);
    }

    pub fn verify_transaction(&self, tx: &Transaction) -> bool {
        if tx.is_coinbase() {
            return true
        }
        let mut prev_txs = HashMap::new();

        for vin in tx.vin.iter() {
            let prev_tx_result = self.find_transaction(&vin.txid);
            if let Ok(prev_tx) = prev_tx_result {
                prev_txs.insert(hex::encode(&prev_tx.id), prev_tx);
            }
        }

        tx.verify(&prev_txs)
    }
}
