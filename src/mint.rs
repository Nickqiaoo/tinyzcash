use crate::{coin::Coin, wallets::Wallets};

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

    let c = Coin::new(wallet.public_key, value);
    let k = c.get_k();
    (
        c,
        MintTransaction {
            id: vec![],
            vout: TXMint {
                v: value,
                k,
                cm: c.cm(),
            },
        },
    )
}

pub fn verify_mint(tx: &MintTransaction) -> bool {
    let cm = Coin::get_cm(&tx.vout.k, tx.vout.v);
    cm == tx.vout.cm
}
