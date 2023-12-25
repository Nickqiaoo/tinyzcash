use std::fmt;
use serde::{Deserialize, Serialize};
use orchard::{
    Action as oAction,
    bundle::{Authorized},
};
use orchard::bundle::Authorization;
use crate::transaction::Transaction;

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Bundle {
    actions:Vec<Action>,
    flags: u8,
    value_balance: i64,
    anchor: String,
    proof: String,
    binding_sig:String,
}
#[derive(Clone,Serialize, Deserialize)]
pub struct Action{
    nullifier: String,
    rk: String,
    cmx:  String,
    out_ciphertext:String,
    ephemeral_key:String,
    enc_ciphertext:String,
    cv: String,
    spend_auth_sig: String,
}

impl From<&oAction<<Authorized as Authorization>::SpendAuth>> for Action {
    fn from(a: &oAction<<Authorized as Authorization>::SpendAuth>) -> Self {
        let  rk:[u8; 32] = a.rk().into();
        let sig :[u8; 64] = a.authorization().into();
        Action{
                nullifier: hex::encode(a.nullifier().to_bytes()),
                rk: hex::encode(rk),
                cmx: hex::encode(a.cmx().to_bytes()),
                out_ciphertext: hex::encode(a.encrypted_note().out_ciphertext),
                ephemeral_key: hex::encode(a.encrypted_note().epk_bytes),
                enc_ciphertext: hex::encode(a.encrypted_note().enc_ciphertext),
                cv: hex::encode(a.cv_net().to_bytes()),
                spend_auth_sig: hex::encode(sig),
            }
    }
}

impl From<&orchard::Bundle<Authorized, i64>> for Bundle {
    fn from(b: &orchard::Bundle<Authorized, i64>) -> Self {
        let sig :[u8; 64] = b.authorization().binding_signature().into();

        Bundle{
            actions: b.actions().iter().map(|action| Action::from(action)).collect(),
            flags: b.flags().to_byte(),
            value_balance: b.value_balance().clone(),
            anchor: hex::encode(b.anchor().to_bytes()),
            proof: hex::encode(b.authorization().proof().as_ref()),
            binding_sig: hex::encode(sig),
        }
    }
}

// impl fmt::Display for Bundle {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         _ = writeln!(f, "{}", hex::encode(&self.id));
//         for (i, v) in self.vin.iter().enumerate() {
//             _ = writeln!(f, "vin{}>>>{}", i, v);
//         }
//         for (i, v) in self.vout.iter().enumerate() {
//             _ = writeln!(f, "vout{}>>>{}", i, v);
//         }
//         Ok(())
//     }
// }