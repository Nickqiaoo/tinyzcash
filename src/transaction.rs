use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use crate::{transaction_output::TXOutput, blockchain::Blockchain, transaction_input::TXInput};

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub id: Vec<u8>,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl Transaction {
    fn set_id(&mut self) {
        let json_str = serde_json::to_string(self).unwrap();
        self.id = Sha256::digest(json_str.as_bytes()).to_vec();
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }
}

// pub fn new_utxo_transaction(from:String, to:String, amount:i64, bc:&Blockchain) -> Transaction{
    
// }

pub fn new_coinbase_tx(to: &str, data:&str) -> Transaction {
    let txin = TXInput { 
        txid: vec![],
        vout: -1,
        script_sig: String::from(data)
    };
    let txout = TXOutput { value: 10, script_pub_key: to.to_string() };
    let mut tx = Transaction { id: vec![], vin: vec![txin], vout: vec![txout] };
    tx.set_id();

    tx
}