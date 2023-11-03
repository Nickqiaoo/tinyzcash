use std::fmt;

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
    pub fn use_key(&self, pub_key_hash: &Vec<u8>) -> bool {
        let locking_hash = hash_pub_key(&self.pub_key);
        locking_hash == *pub_key_hash
    }
}

impl fmt::Display for TXInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "txid:{} vout:{} signature:{} pub_key:{}", hex::encode(&self.txid), self.vout, hex::encode(&self.signature), hex::encode(&self.pub_key))
    }
}