use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fmt, collections::HashMap};

use crate::{blockchain::Blockchain, transaction_input::TXInput, transaction_output::TXOutput, wallets::Wallets, wallet};

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
        let mut tx_copy = self.clone();
        tx_copy.id = vec![];

        Sha256::digest(tx_copy.serialize().as_bytes()).to_vec()
    }

    fn trimmed_copy(&self) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for vin in &self.vin {
            inputs.push(TXInput {
                txid: vin.txid.clone(),
                vout: vin.vout,
                signature: vec![],
                pub_key: vec![],
            });
        }

        for vout in &self.vout {
            outputs.push(TXOutput {
                value: vout.value,
                pub_key_hash: vout.pub_key_hash.clone(),
            });
        }

        Transaction {
            id: self.id.clone(),
            vin: inputs,
            vout: outputs,
        }
    }

    pub fn sign(&mut self, private_key: secp256k1::SecretKey, prev_txs: &HashMap<String, Transaction>) {
        if self.is_coinbase() {
            return;
        }

        for vin in &self.vin {
            if prev_txs
                .get(&hex::encode(vin.txid.as_slice()))
                .unwrap()
                .id
                .is_empty()
            {
                panic!("ERROR: Previous transaction is not correct");
            }
        }
        let mut tx_copy = self.trimmed_copy();
        
        for in_id in 0..tx_copy.vin.len() {
            let mut vin = tx_copy.vin[in_id].clone();
            let prev_tx = prev_txs.get(&hex::encode(&vin.txid)).unwrap();
            vin.signature = vec![];
            vin.pub_key = prev_tx.vout[vin.vout as usize].pub_key_hash.clone();
            
            tx_copy.vin[in_id] = vin;
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[in_id].pub_key = vec![];
        
            let tx_copy_message = secp256k1::Message::from_slice(&tx_copy.id).unwrap();
            let context = secp256k1::Secp256k1::new();
            let signature = context.sign(&tx_copy_message, &private_key);
            let sig = signature.serialize_compact();
        
            tx_copy.vin[in_id].signature = sig.to_vec();
        }
        
        // for (in_id, vin) in tx_copy.vin.iter_mut().enumerate() {
        //     let prev_tx = prev_txs.get(&hex::encode(&vin.txid)).unwrap();
        //     vin.signature = vec![];
        //     vin.pub_key = prev_tx.vout[vin.vout as usize].pub_key_hash.clone();
        //     tx_copy.id = tx_copy.hash();
        //     vin.pub_key = vec![];

        //     let tx_copy_message = secp256k1::Message::from_slice(&tx_copy.id).unwrap();
        //     let context = secp256k1::Secp256k1::new();
        //     let signature = context.sign(&tx_copy_message, &private_key);
        //     let sig = signature.serialize_compact();

        //     self.vin[in_id].signature = sig.to_vec();
        // }
    }

    pub fn verify(&self, prev_txs: &HashMap<String, Transaction>) -> bool {
        let mut tx_copy = self.trimmed_copy();
        let secp = secp256k1::Secp256k1::new();
        
        for (in_id, vin) in self.vin.iter().enumerate() {
            let prev_tx = &prev_txs[&hex::encode(&vin.txid)];
            tx_copy.vin[in_id].signature = vec![];
            tx_copy.vin[in_id].pub_key = prev_tx.vout[vin.vout as usize].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[in_id].pub_key = vec![];
            
            let tx_copy_message = secp256k1::Message::from_slice(&tx_copy.id).unwrap();
    
            let pk = secp256k1::PublicKey::from_slice(&vin.pub_key).unwrap();
    
            let sig = secp256k1::Signature::from_compact(&vin.signature).unwrap();
            if !secp.verify(&tx_copy_message, &sig, &pk).is_ok() {
                return false;
            }
        }
    
        true
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        _ = write!(f, "{}\n", hex::encode(&self.id));
        for (i, v) in self.vin.iter().enumerate() {
            _ = write!(f, "vin{}>>>{}\n", i, v);
        }
        for (i, v) in self.vout.iter().enumerate() {
            _ = write!(f, "vout{}>>>{}\n", i, v);
        }
        Ok(())
    }
}

pub fn new_coinbase_tx(to: &str, data: &str) -> Transaction {
    let txin = TXInput {
        txid: vec![],
        vout: -1,
        signature: vec![],
        pub_key: data.as_bytes().to_vec(),
    };
    let txout = TXOutput::new(10, to);
    let mut tx = Transaction {
        id: vec![],
        vin: vec![txin],
        vout: vec![txout],
    };
    tx.set_id();

    tx
}

pub fn new_utxo_transaction(from: String, to: String, amount: i64, bc: &Blockchain) -> Transaction {
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();

    let wallets = Wallets::new();
    let wallet = wallets.get_wallet(&from).unwrap();
    let pub_key_hash = wallet::hash_pub_key(wallet.public_key.as_bytes());

    let (acc, valid_outputs) = bc.find_spendable_outputs(&pub_key_hash, amount);
    if acc < amount {
        panic!("ERROR: Not enough funds");
    }

    for (txid, outs) in valid_outputs {
        let tx_id = hex::decode(txid.clone()).unwrap();
        for out in outs {
            let input = TXInput {
                txid: tx_id.clone(),
                vout: out,
                signature: vec![],
                pub_key: wallet.public_key.clone().into_bytes(),
            };
            inputs.push(input);
        }
    }

    outputs.push(TXOutput::new(amount, &to));
    if acc > amount {
        outputs.push(TXOutput::new(acc - amount, &from));
    }

    let mut tx = Transaction {
        vin: inputs,
        vout: outputs,
        id: Vec::new(),
    };
    tx.set_id();
    bc.sign_transaction(&mut tx, wallet.private_key.clone());
    tx
}
