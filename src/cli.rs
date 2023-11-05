use std::{println, vec};

use crate::{blockchain::Blockchain, pow::ProofOfWork, transaction, wallet, wallets::Wallets};
use structopt::StructOpt;

pub struct CLI {
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "blockchain", about = "A simple CLI application")]
pub enum Command {
    #[structopt(name = "createBlockchain", about = "CreateBlockChain")]
    CreateBlockChain {
        #[structopt(help = "Address")]
        address: String,
    },

    #[structopt(name = "createwallet", about = "create wallet")]
    Createwallet,

    #[structopt(name = "printchain", about = "Print the chain")]
    PrintChain,

    #[structopt(name = "listaddress", about = "ListAddress")]
    ListAddress,

    #[structopt(name = "send", about = "send")]
    Send {
        #[structopt(help = "from")]
        from: String,
        #[structopt(help = "to")]
        to: String,
        #[structopt(help = "amount")]
        amount: i64,
    },

    #[structopt(name = "getbalance", about = "getbalance")]
    Getbalance {
        #[structopt(help = "Address")]
        address: String,
    },
}

impl CLI {
    pub fn run(&mut self) {
        match &self.cmd {
            Command::CreateBlockChain { address } => self.create_blockchain(address.to_string()),
            Command::Createwallet => self.create_wallet(),
            Command::PrintChain => self.print_chain(),
            Command::ListAddress => self.list_address(),
            Command::Send { from, to, amount } => {
                self.send(from.to_string(), to.to_string(), amount.clone())
            }
            Command::Getbalance { address } => self.get_balance(address.to_string()),
        }
    }

    fn create_blockchain(&self, address: String) {
        Blockchain::new(address.as_str());
        println!("Done");
    }

    fn create_wallet(&self) {
        let mut w = Wallets::new();
        w.create_wallet();
        _ = w.save_to_file();
    }

    fn list_address(&self) {
        let w = Wallets::new();
        let address = w.get_addresses();
        for a in &address {
            println!("{:}", a);
        }
    }

    fn print_chain(&self) {
        let bc = Blockchain::new("");
        let mut bci = bc.iterator();

        loop {
            if let Some(block) = bci.next() {
                println!("Prev hash: {:}", hex::encode(&block.prev_block_hash));
                println!("Hash: {:}", hex::encode(&block.hash));
                let pow = ProofOfWork::new(&block);
                println!("PoW: {:}", pow.validate());
                println!("Transactions:");
                for (i, tx) in block.transactions.iter().enumerate() {
                    println!("tx{:}: {:}", i, tx);
                }
                println!();
            } else {
                break;
            }
        }
    }

    fn send(&self, from: String, to: String, amount: i64) {
        if !wallet::validate_address(&from) {
            panic!("Sender address is not valid")
        }
        if !wallet::validate_address(&to) {
            panic!("Recipient address is not valid")
        }
        let mut bc = Blockchain::new(&from);
        let tx = transaction::new_utxo_transaction(from.to_string(), to.to_string(), amount, &bc);
        bc.mine_block(vec![tx]);
        println!("Success!");
    }

    fn get_balance(&self, address: String) {
        if !wallet::validate_address(&address) {
            panic!("address is not valid")
        }
        let bc = Blockchain::new(&address);
        let mut balance = 0;
        let mut pub_key_hash = bs58::decode(address.clone()).into_vec().unwrap();
        pub_key_hash = pub_key_hash[1..pub_key_hash.len() - wallet::CHECKSUM_LENGTH].to_vec();
        let utxos = bc.find_utxo(&pub_key_hash);
        for out in utxos {
            balance += out.value;
        }
        println!("Balance of '{}': {}", address, balance);
    }
}
