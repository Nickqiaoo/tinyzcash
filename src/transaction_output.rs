use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TXOutput {
    pub value: i64,
    pub pub_key_hash: Vec<u8>,
}

impl TXOutput {
    pub fn new(value: i64, address: &str) -> Self {
        let mut txo = TXOutput {
            value,
            pub_key_hash: vec![],
        };
        txo.lock(address.as_bytes().to_vec());
        txo
    }

    pub fn lock(&mut self, address: Vec<u8>) {
        let pub_key_hash = bs58::decode(&address).into_vec().unwrap();
        self.pub_key_hash = pub_key_hash[1..pub_key_hash.len() - 4].to_vec();
    }

    pub fn is_locked_with_key(&self, pub_key_hash: &Vec<u8>) -> bool {
        self.pub_key_hash == *pub_key_hash
    }
}

impl fmt::Display for TXOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "value:{} pub_key_hash:{}",
            self.value,
            hex::encode(&self.pub_key_hash)
        )
    }
}
