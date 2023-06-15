use std::collections::HashMap;

use crate::wallet::Wallet;

pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

fn new_wallets() -> Result<Wallets, Box<dyn std::error::Error>> {
    let mut wallets = Wallets::default();
    wallets.wallets = HashMap::new();

    let err = wallets.load_from_file()?;

    Ok(wallets)
}

impl Wallets {
    fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        let address = format!("{:?}", wallet.get_address());

        self.wallets.insert(address.clone(), wallet);

        address
    }

    fn get_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::new();

        for address in self.wallets.keys() {
            addresses.push(address.clone());
        }

        addresses
    }

    fn get_wallet(&self, address: &str) -> &Wallet {
        &self.wallets[address]
    }
}

