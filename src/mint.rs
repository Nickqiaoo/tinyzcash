use secp256k1::{PublicKey, Secp256k1};
use sha2::{Digest, Sha256};

use crate::{util, wallets::Wallets};

pub struct Coin {
    addr_pk: String,
    rho: Vec<u8>,
    v: u64,
    r: Vec<u8>,
    cm: Vec<u8>,
}

pub struct TXMint {
    v: u64,
    k: Vec<u8>,
    cm: Vec<u8>,
}

pub struct MintTransaction {
    pub id: Vec<u8>,
    pub vout: TXMint,
}

pub fn mint(address: String, value: u64) -> (Coin, MintTransaction) {
    let wallets = Wallets::new();
    let wallet = wallets.get_wallet(&address).unwrap();

    let rho = util::generate_random_bytes(256);
    let r = util::generate_random_bytes(384);

    let k = create_k(wallet.public_key, &rho, &r);
    let cm = create_cm(&k, value);

    let c = Coin {
        addr_pk: wallet.public_key,
        rho,
        v: value,
        r,
        cm,
    };
    (
        c,
        MintTransaction {
            id: vec![],
            vout: TXMint { v: value, k, cm },
        },
    )
}

//H(r || H(pk || rho))
fn create_k(public_key: String, rho: &Vec<u8>, r: &Vec<u8>) -> Vec<u8> {
    let public_key_bytes = hex::decode(public_key).expect("Failed to decode public key");
    let secp = Secp256k1::new();
    let public_key = PublicKey::from_slice(&public_key_bytes).expect("Invalid public key");

    let mut combined_data = Vec::new();
    combined_data.extend_from_slice(&public_key.serialize_uncompressed());
    combined_data.extend_from_slice(&rho);

    let midk = Sha256::digest(&combined_data).to_vec();
    let truncated_hash: Vec<u8> = midk.iter().take(128 / 8).cloned().collect();

    combined_data = Vec::new();
    combined_data.extend_from_slice(&truncated_hash);
    combined_data.extend_from_slice(&r);

    Sha256::digest(&combined_data).to_vec()
}

fn create_cm(k: &Vec<u8>, v: u64) -> Vec<u8> {
    let zero_padding: Vec<u8> = vec![0; 192 / 8];

    let mut combined_data = Vec::new();
    combined_data.extend_from_slice(&k);
    combined_data.extend_from_slice(&zero_padding);
    combined_data.extend_from_slice(&v.to_be_bytes());

    Sha256::digest(&combined_data).to_vec()
}

pub fn verify_mint(tx: &MintTransaction) -> bool {
    let cm = create_cm(&tx.vout.k, tx.vout.v);
    cm == tx.vout.cm
}
