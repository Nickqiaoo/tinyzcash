use secp256k1::{Secp256k1, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use ripemd160::{Ripemd160, Digest as RipemdDigest};
use rand::Rng;
use bs58;

const VERSION: u8 = 0x00;
pub(crate) const CHECKSUM_LENGTH: usize = 4;

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
}

impl Wallet {
    pub fn new() -> Wallet {
        let (private_key, public_key) = new_key_pair();
        
        Wallet {private_key: private_key.to_string(), public_key:  public_key.to_string()}
    }

    pub fn get_address(&self) -> String {
        let pub_key_hash = hash_pub_key(self.public_key.as_bytes());
        let mut versioned_payload = vec![VERSION];
        versioned_payload.extend_from_slice(&pub_key_hash);
        let checksum = checksum(&versioned_payload);
        versioned_payload.extend_from_slice(&checksum);
        bs58::encode(&versioned_payload).into_string()
    }
}

pub fn validate_address(address: &String) -> bool {
    let pub_key_hash = bs58::decode(address).into_vec().unwrap();
    let actual_checksum = &pub_key_hash[pub_key_hash.len() - CHECKSUM_LENGTH..];
    let version = pub_key_hash[0];
    let pub_key_hash = &pub_key_hash[1..pub_key_hash.len() - CHECKSUM_LENGTH];
    let target_checksum = checksum([&[version], pub_key_hash].concat().as_slice());

    actual_checksum == target_checksum
}

pub fn new_key_pair() -> (SecretKey, PublicKey) {
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

pub fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
    let pub_key_sha256 = Sha256::digest(pub_key);

    let mut ripemd160 = Ripemd160::new();
    ripemd160.input(pub_key_sha256);
    ripemd160.result().to_vec()
}

pub fn checksum(payload: &[u8]) -> Vec<u8> {
    let first_sha = Sha256::digest(payload);

    let second_sha = Sha256::digest(&first_sha.as_slice());

    second_sha[0..CHECKSUM_LENGTH].to_vec()
}