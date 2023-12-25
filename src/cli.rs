use std::{println, vec};

use crate::transaction::new_coinbase_tx;
use crate::{
    blockchain::Blockchain, deposit, pow::ProofOfWork, transaction, verify, wallet,
    wallets::Wallets, withdraw, zsend,
};
use structopt::StructOpt;

pub struct Cli {
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "tinyzcash", about = "A simple CLI application")]
pub enum Command {
    #[structopt(name = "createblockchain", about = "createblockchain")]
    CreateBlockChain {
        #[structopt(help = "Address")]
        address: String,
    },

    #[structopt(name = "createwallet", about = "create wallet")]
    Createwallet,

    #[structopt(name = "printchain", about = "print the chain")]
    PrintChain,

    #[structopt(name = "listaddress", about = "listAddress")]
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
    #[structopt(name = "deposit", about = "deposit")]
    Deposit {
        #[structopt(help = "address")]
        address: String,
        #[structopt(help = "amount")]
        amount: u64,
    },
    #[structopt(name = "zsend", about = "zsend")]
    Zsend {
        #[structopt(help = "from")]
        from: String,
        #[structopt(help = "to")]
        to: String,
    },
    #[structopt(name = "withdraw", about = "withdraw")]
    Withdraw {
        #[structopt(help = "address")]
        address: String,
    },
}

impl Cli {
    pub fn run(&mut self) {
        match &self.cmd {
            Command::CreateBlockChain { address } => self.create_blockchain(address.clone()),
            Command::Createwallet => self.create_wallet(),
            Command::PrintChain => self.print_chain(),
            Command::ListAddress => self.list_address(),
            Command::Send { from, to, amount } => self.send(from.clone(), to.clone(), *amount),
            Command::Getbalance { address } => self.get_balance(address.clone()),
            Command::Deposit { address, amount } => self.deposit(address.clone(), *amount),
            Command::Zsend { from, to } => self.zsend(from.clone(), to.clone()),
            Command::Withdraw { address } => self.withdraw(address.clone()),
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
            println!("addr:{:}", a);
        }
        let address = w.get_z_addresses();
        for a in &address {
            println!("zaddr:{:}", a);
        }
    }

    fn print_chain(&self) {
        let bc = Blockchain::new("");
        let mut bci = bc.iterator();

        while let Some(block) = bci.next() {
            println!("Prev hash: {:}", hex::encode(&block.prev_block_hash));
            println!("Hash: {:}", hex::encode(&block.hash));
            let pow = ProofOfWork::new(&block);
            println!("PoW: {:}", pow.validate());
            println!("Transactions:");
            for (i, tx) in block.transactions.iter().enumerate() {
                println!("tx{:}: {:}", i, tx);
            }
            println!();
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
        let tx = transaction::new_utxo_transaction(from, to, amount, &bc);
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

    fn deposit(&self, address: String, amount: u64) {
        let mut bc = Blockchain::new(&address);
        let mut tx = transaction::new_utxo_transaction(
            address.clone(),
            "11111111111111111111".to_string(),
            amount as i64,
            &bc,
        );

        let bundle = deposit::deposit(&address, amount);
        verify::verify_bundle(&bundle);

        tx.bundle = (&bundle).into();
        bc.mine_block(vec![tx]);
        deposit::save_note(&bundle, &address);
    }

    fn zsend(&self, from: String, to: String) {
        let mut bc = Blockchain::new(&from);

        let bundle = zsend::zsend(&from, &to);
        verify::verify_bundle(&bundle);

        let mut tx = transaction::Transaction {
            bundle: (&bundle).into(),
            ..Default::default()
        };
        tx.set_id();
        bc.mine_block(vec![tx]);
        zsend::save_note(&bundle, &from, &to);
    }
    fn withdraw(&self, address: String) {
        let mut bc = Blockchain::new(&address);

        let bundle = withdraw::withdraw(&address);
        verify::verify_bundle(&bundle);

        let wallets = Wallets::new();
        let wallet = wallets.get_z_wallet(&address).unwrap();
        let mut tx = new_coinbase_tx(&wallet.get_address(), "withdraw", *bundle.value_balance());
        tx.bundle = (&bundle).into();
        bc.mine_block(vec![tx]);
        withdraw::save_note(&address);
    }
}
