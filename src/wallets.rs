use crate::wallet::Wallet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

const WALLET_FILE: &str = "wallets.dat";

#[derive(Serialize, Deserialize)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
    zwallets:HashMap<String, Wallet>,
}

impl Wallets {
    pub fn new() -> Self {
        Self::load_from_file().unwrap()
    }

    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = wallet.get_address();
        let zaddr = wallet.get_z_address();

        self.wallets.insert(address.clone(), wallet.clone());
        self.zwallets.insert(zaddr.clone(), wallet);

        address
    }

    pub fn get_addresses(&self) -> Vec<String> {
        self.wallets.keys().cloned().collect()
    }

    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    pub fn get_z_wallet(&self, address: &str) -> Option<&Wallet> {
        self.zwallets.get(address)
    }

    fn load_from_file() -> io::Result<Self> {
        if Path::new(WALLET_FILE).exists() {
            let mut file = File::open(WALLET_FILE)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            let wallets: Wallets = serde_json::from_str(&content)?;

            Ok(wallets)
        } else {
            Ok(Wallets {
                wallets: HashMap::new(),
                zwallets: HashMap::new(),
            })
        }
    }

    pub fn save_to_file(&self) -> io::Result<()> {
        let content = serde_json::to_string(self)?;

        fs::write(WALLET_FILE, content)
    }
}
