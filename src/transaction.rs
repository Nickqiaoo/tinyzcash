use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::fmt;

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

    pub fn serialize(&self) -> String {
        let json_str = serde_json::to_string(self).unwrap();
        json_str
    }

    fn hash(&self) -> Vec<u8> {
        let mut hash = [0; 32];
    
        let mut tx_copy = self.clone();
        tx_copy.id = vec![];
    
        hash = sha256::hash(tx_copy.serialize().as_slice());
    
        hash.to_vec()
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}


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

pub fn new_utxo_transaction(from:String, to:String, amount:i64, bc:&Blockchain) -> Transaction{
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    
    let (acc, valid_outputs) = bc.find_spendable_outputs(from.as_str(), amount);
    if acc < amount {
        panic!("ERROR: Not enough funds");
    }

    // Build a list of inputs
    for (txid, outs) in valid_outputs {

        for out in outs {
            let input = TXInput { txid: hex::decode(txid.clone()).unwrap(), vout:out, script_sig: from.to_string(), };
            inputs.push(input);
        }
    }

    // Build a list of outputs
    outputs.push(TXOutput { value: amount, script_pub_key: to.to_string() });
    if acc > amount {
        outputs.push(TXOutput { value: acc - amount, script_pub_key: from.to_string() }); // a change
    }

    let mut tx = Transaction { vin: inputs, vout: outputs, id:Vec::new()};
    tx.set_id();

    tx
}