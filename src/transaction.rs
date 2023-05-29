use num::BigUint;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fmt, collections::HashMap};

use crate::{blockchain::Blockchain, transaction_input::TXInput, transaction_output::TXOutput};

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

    fn trimmed_copy(&self) -> Transaction {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for vin in &self.vin {
            inputs.push(TXInput {
                txid: vin.txid.clone(),
                vout: vin.vout,
                signature: None,
                pub_key: None,
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
                .is_none()
            {
                panic!("ERROR: Previous transaction is not correct");
            }
        }
        let mut tx_copy = tx.trimmed_copy();

        for (in_id, vin) in tx_copy.vin.iter_mut().enumerate() {
            let prev_tx = prev_txs.get(&vin.txid).unwrap();
            vin.signature = None;
            vin.pub_key = prev_tx.vout[vin.vout].pub_key_hash.clone();

            let tx_copy_message = secp256k1::Message::from_slice(&tx_copy.hash()).unwrap();
            let context = secp256k1::Secp256k1::new();

            let mut rng = secp256k1::OsRng;
            let (signature, _) = context.sign(&tx_copy_message, private_key, &mut rng);
            let (r, s) = signature.serialize();

            let mut signature_vec = Vec::new();
            signature_vec.extend_from_slice(&r[..]);
            signature_vec.extend_from_slice(&s[..]);

            self.vin[in_id].signature = Some(signature_vec);
            tx_copy.vin[in_id].pub_key = None;
        }
    }

    pub fn verify(&self, prev_txs: &HashMap<String, Transaction>) -> bool {
        let mut tx_copy = self.trimmed_copy();
        let secp = secp256k1::Secp256k1::new();
        
        for (in_id, vin) in self.vin.iter().enumerate() {
            let prev_tx = &prev_txs[&hex::encode(&vin.txid)];
            tx_copy.vin[in_id].signature = None;
            tx_copy.vin[in_id].pub_key = prev_tx.vout[vin.vout as usize].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[in_id].pub_key = None;
            
            let r = BigUint::from_bytes_be(&vin.signature[..(vin.signature.len() / 2)]);
            let s = BigUint::from_bytes_be(&vin.signature[(vin.signature.len() / 2)..]);
            let x = BigUint::from_bytes_be(&vin.pub_key[..(vin.pub_key.len() / 2)]);
            let y = BigUint::from_bytes_be(&vin.pub_key[(vin.pub_key.len() / 2)..]);
    
            let pk = PublicKey::from_slice(&secp, &x.to_bytes_be(), &y.to_bytes_be()).unwrap();
    
            let sig = Signature::from_der(&secp, &vin.signature).unwrap();
            //let pubkey = secp256k1::PublicKey::from_slice(&input.pub_key).unwrap();
            //let signature = secp256k1::Signature::from_der(&input.signature).unwrap();
            if !secp.verify(&tx_copy.id.into_inner(), &sig, &pk).is_ok() {
                return false;
            }
        }
    
        true
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

pub fn new_coinbase_tx(to: &str, data: &str) -> Transaction {
    let txin = TXInput {
        txid: vec![],
        vout: -1,
        script_sig: String::from(data),
    };
    let txout = TXOutput {
        value: 10,
        script_pub_key: to.to_string(),
    };
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

    let (acc, valid_outputs) = bc.find_spendable_outputs(from.as_str(), amount);
    if acc < amount {
        panic!("ERROR: Not enough funds");
    }

    // Build a list of inputs
    for (txid, outs) in valid_outputs {
        for out in outs {
            let input = TXInput {
                txid: hex::decode(txid.clone()).unwrap(),
                vout: out,
                script_sig: from.to_string(),
            };
            inputs.push(input);
        }
    }

    // Build a list of outputs
    outputs.push(TXOutput {
        value: amount,
        script_pub_key: to.to_string(),
    });
    if acc > amount {
        outputs.push(TXOutput {
            value: acc - amount,
            script_pub_key: from.to_string(),
        }); // a change
    }

    let mut tx = Transaction {
        vin: inputs,
        vout: outputs,
        id: Vec::new(),
    };
    tx.set_id();

    tx
}
