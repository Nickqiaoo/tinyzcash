use rand::Rng;
use secp256k1::{PublicKey, Secp256k1};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct Coin {
    addr_pk: String,
    pub rho: Vec<u8>,
    pub v: u64,
    pub r: Vec<u8>,
    cm: Vec<u8>,
}

impl Coin {
    pub fn new(public_key: &String, value: u64) -> Self {
        let rho = generate_random_bytes(256);
        let r = generate_random_bytes(384);

        let k = Self::get_k_inner(public_key, &rho, &r);
        let cm = Self::get_cm(&k, value);
        Coin {
            addr_pk: public_key.clone(),
            rho,
            v: value,
            r,
            cm,
        }
    }
    pub fn get_k(&self) -> Vec<u8> {
        Self::get_k_inner(&self.addr_pk, &self.rho, &self.r)
    }

    //H(r || H(pk || rho))
    fn get_k_inner(public_key: &String, rho: &Vec<u8>, r: &Vec<u8>) -> Vec<u8> {
        let public_key_bytes = hex::decode(public_key).expect("Failed to decode public key");
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_slice(&public_key_bytes).expect("Invalid public key");

        let mut combined_data = Vec::new();
        combined_data.extend_from_slice(&public_key.serialize_uncompressed());
        combined_data.extend(rho);

        let midk = Sha256::digest(&combined_data).to_vec();
        let truncated_hash: Vec<u8> = midk.iter().take(128 / 8).cloned().collect();

        combined_data = Vec::new();
        combined_data.extend(truncated_hash);
        combined_data.extend(r);

        Sha256::digest(&combined_data).to_vec()
    }

    pub fn cm(&self) -> Vec<u8> {
        self.cm.clone()
    }

    pub fn get_cm(k: &Vec<u8>, v: u64) -> Vec<u8> {
        let zero_padding: Vec<u8> = vec![0; 192 / 8];

        let mut combined_data = Vec::new();
        combined_data.extend(k);
        combined_data.extend(zero_padding);
        combined_data.extend_from_slice(&v.to_be_bytes());

        Sha256::digest(&combined_data).to_vec()
    }
}

pub fn generate_random_bytes(bits: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();

    let byte_count = (bits + 7) / 8;

    let random_bytes: Vec<u8> = (0..byte_count).map(|_| rng.gen()).collect();

    random_bytes
}
