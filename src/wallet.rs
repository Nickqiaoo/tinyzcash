use orchard::keys::FullViewingKey;
use orchard::note::{Nullifier, RandomSeed};
use orchard::value::NoteValue;
use orchard::{keys, Address};
use rand::rngs::OsRng as randRng;
use rand::RngCore;
use ripemd::{Digest as RipemdDigest, Ripemd160};
use secp256k1::rand::rngs::OsRng;
use secp256k1::Secp256k1;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

const VERSION: u8 = 0x00;
pub(crate) const CHECKSUM_LENGTH: usize = 4;

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
    pub spend_key: String,
    pub notes: Vec<Note>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Note {
    pub value: u64,
    pub rseed: [u8; 32],
    pub nf: [u8; 32],
}

impl Note {
    pub fn to_note(&self, addr: orchard::Address) -> orchard::Note {
        let old_nf = Nullifier::from_bytes(&self.nf).unwrap();
        orchard::Note::from_parts(
            addr,
            NoteValue::from_raw(self.value),
            old_nf,
            RandomSeed::from_bytes(self.rseed, &old_nf).unwrap(),
        )
        .unwrap()
    }
}

impl Wallet {
    pub fn new() -> Wallet {
        let secp = Secp256k1::new();
        let (private_key, public_key) = secp.generate_keypair(&mut OsRng);

        let mut rng = randRng;
        let mut random_bytes = [0u8; 32];
        rng.fill_bytes(&mut random_bytes);
        let spend_key = keys::SpendingKey::from_zip32_seed(&random_bytes, 0, 0).unwrap();

        Wallet {
            private_key: hex::encode(private_key.secret_bytes()),
            public_key: public_key.to_string(),
            spend_key: hex::encode(spend_key.to_bytes()),
            notes: vec![],
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

    pub fn get_z_address(&self) -> String {
        let addr = self.z_address();
        hex::encode(addr.to_raw_address_bytes())
    }

    pub fn sk(&self) -> keys::SpendingKey {
        let spend_key = hex::decode(&self.spend_key).unwrap();
        let spend_key: Result<[u8; 32], _> = spend_key.try_into();
        keys::SpendingKey::from_bytes(spend_key.unwrap()).unwrap()
    }

    pub fn z_address(&self) -> Address {
        let fvk = FullViewingKey::from(&self.sk());
        fvk.address_at(0u32, keys::Scope::External)
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
