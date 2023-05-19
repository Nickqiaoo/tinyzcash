use secp256k1::{Secp256k1, PublicKey, SecretKey};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use ripemd160::{Ripemd160, Digest as RipemdDigest};
use rand::Rng;
use bs58;

const VERSION: u8 = 0x00;
const CHECKSUM_LENGTH: usize = 4;

pub struct Wallet {
    private_key: SecretKey,
    public_key: PublicKey,
}

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallet {
    pub fn new() -> Wallet {
        let (private_key, public_key) = Self::new_key_pair();
        Wallet { private_key, public_key }
    }

    fn new_key_pair() -> (SecretKey, PublicKey) {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        loop {
            let private_key_candidate = SecretKey::from_slice(&rng.gen::<[u8; 32]>());
            if let Ok(private_key) = private_key_candidate {
                let public_key = PublicKey::from_secret_key(&secp, &private_key);
                return (private_key, public_key);
            }
        }
    }

    pub fn get_address(&self) -> Vec<u8> {
        let pub_key_hash = Self::hash_pub_key(&self.public_key.serialize());
        let mut versioned_payload = vec![VERSION];
        versioned_payload.extend_from_slice(&pub_key_hash);
        let checksum = Self::checksum(&versioned_payload);
        versioned_payload.extend_from_slice(&checksum);
        bs58::encode(&versioned_payload).into_vec()
    }

    fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
        let pub_key_sha256 = Sha256::digest(pub_key);

        let mut ripemd160 = Ripemd160::new();
        ripemd160.input(pub_key_sha256);
        ripemd160.result().to_vec()
    }

    fn checksum(payload: &[u8]) -> Vec<u8> {
        let first_sha = Sha256::digest(payload);

        let second_sha = Sha256::digest(&first_sha.as_slice());

        second_sha[0..CHECKSUM_LENGTH].to_vec()
    }
}
