use serde::{Deserialize, Serialize};

use crate::coin::Coin;

#[derive(Serialize, Deserialize)]
pub struct WitnessX {
    pub rt: Vec<u8>,
    pub old_sn: Vec<u8>,
    pub new_cm: Vec<u8>,
    pub public_value: u64,
    pub h_sig: Vec<u8>,
    pub h: Vec<u8>,
}

pub struct WitnessA {
    pub path: Vec<Vec<u8>>,
    pub old_coin: Coin,
    pub secret_key: String,
    pub new_coin: Coin,
}

pub fn create_proof(x: WitnessX, a: WitnessA) -> Vec<u8> {}
