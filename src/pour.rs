use crate::{
    circuit::{self, WitnessA, WitnessX},
    coin::Coin,
    wallet::Wallet,
    wallets::Wallets,
};
use ecies::{decrypt, encrypt};
use sha2::{Digest, Sha256};

pub struct TXPour {
    rt: Vec<u8>,
    old_sn: Vec<u8>,
    new_cm: Vec<u8>,
    public_value: u64,
    info: String,
    pk_sig: String,
    h: Vec<u8>,
    pi_pour: Vec<u8>,
    c_info: Vec<u8>,
    sigma: Vec<u8>,
}

pub struct PourTransaction {
    pub id: Vec<u8>,
    pub vout: TXPour,
}

pub fn pour(
    merkle_root: &Vec<u8>,
    old_coin: &Coin,
    old_adress: String,
    merkle_path: &Vec<Vec<u8>>,
    new_value: u64,
    new_address: String,
    public_value: u64,
    info: String,
) -> (Coin, PourTransaction) {
    let wallets = Wallets::new();
    let wallet_new = wallets.get_wallet(&new_address).unwrap();
    let wallet_old = wallets.get_wallet(&old_adress).unwrap();

    let sn_msg = wallet_old.private_key.as_bytes().to_vec();
    sn_msg.extend(old_coin.rho);
    let old_sn = Sha256::digest(sn_msg).to_vec();

    let c = Coin::new(wallet_new.public_key, new_value);
    let c_info = create_c_info(&wallet_new.public_key, &c.rho, c.v, &c.r);

    let sig_wallet = Wallet::new();
    let h_sig = Sha256::digest(&sig_wallet.public_key).to_vec();
    
    let h_msg = wallet_old.private_key.as_bytes().to_vec();
    h_msg.extend(h_sig);
    let h = Sha256::digest(h_msg).to_vec();

    let wx = WitnessX {
        rt: merkle_root.clone(),
        old_sn: old_sn.clone(),
        new_cm: c.cm(),
        public_value,
        h_sig: h_sig.clone(),
        h: h.clone(),
    };
    let wa = WitnessA {
        path: merkle_path.clone(),
        old_coin: old_coin.clone(),
        secret_key: wallet_old.private_key.clone(),
        new_coin: c,
    };

    let pi_pour = circuit::create_proof(wx, wa);
    let sigma = create_sig(&sig_wallet.private_key, &wx, &pi_pour, &info, &c_info);

    (
        c,
        PourTransaction {
            id: vec![],
            vout: TXPour {
                rt: merkle_root.clone(),
                old_sn,
                new_cm: c.cm(),
                public_value,
                info,
                pk_sig: sig_wallet.public_key,
                h,
                pi_pour,
                c_info,
                sigma,
            },
        },
    )
}

fn create_c_info(public_key: &String, rho: &Vec<u8>, v: u64, r: &Vec<u8>) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend(rho);
    message.extend(&v.to_be_bytes().to_vec());
    message.extend(r);

    encrypt(public_key.as_bytes(), &message).unwrap()
}

fn create_sig(
    sk: &String,
    x: &WitnessX,
    pi_pour: &Vec<u8>,
    info: &String,
    c_info: &Vec<u8>,
) -> Vec<u8> {
    let priv_key = secp256k1::SecretKey::from_slice(hex::decode(sk).unwrap().as_slice()).unwrap();

    let msg = serde_json::to_string(x).unwrap();
    msg = format!("{}{:?}{}{:?}", msg, pi_pour, info, c_info);
    let sig_message = secp256k1::Message::from_digest_slice(&msg.as_bytes()).unwrap();

    let secp = secp256k1::Secp256k1::new();
    secp.sign_ecdsa(&sig_message, &priv_key)
        .serialize_compact()
        .to_vec()
}
