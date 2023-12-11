use ripemd::{Digest as RipemdDigest, Ripemd160};
use secp256k1::rand::rngs::OsRng;
use secp256k1::Secp256k1;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

const VERSION: u8 = 0x00;
pub(crate) const CHECKSUM_LENGTH: usize = 4;

#[derive(Serialize, Deserialize)]
pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
}

impl Wallet {
    pub fn new() -> Wallet {
        let secp = Secp256k1::new();
        let (private_key, public_key) = secp.generate_keypair(&mut OsRng);

        Wallet {
            private_key: hex::encode(private_key.secret_bytes()),
            public_key: public_key.to_string(),
        }
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

pub fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
    let pub_key_sha256 = Sha256::digest(pub_key);

    Ripemd160::digest(pub_key_sha256).to_vec()
}

pub fn checksum(payload: &[u8]) -> Vec<u8> {
    let first_sha = Sha256::digest(payload);

    let second_sha = Sha256::digest(first_sha.as_slice());

    second_sha[0..CHECKSUM_LENGTH].to_vec()
}
