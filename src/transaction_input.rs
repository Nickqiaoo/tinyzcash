use serde::{Serialize, Deserialize};

use crate::wallet::hash_pub_key;

#[derive(Serialize, Deserialize, Clone)]
pub struct TXInput {
    pub txid: Vec<u8>,
    pub vout: i32,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}


impl TXInput {
    pub fn uses_key(&self, pub_key_hash: Vec<u8>) -> bool {
        let locking_hash = hash_pub_key(&self.pub_key);
        locking_hash == pub_key_hash
    }
}