use std::{println, vec};

use structopt::StructOpt;
use crate::{blockchain::Blockchain, pow::ProofOfWork, transaction::Transaction};

pub struct CLI {
    pub cmd: Command
}

#[derive(StructOpt, Debug)]
#[structopt(name = "blockchain", about = "A simple CLI application")]
pub enum Command {
    #[structopt(name = "createBlockchain", about = "CreateBlockChain")]
    CreateBlockChain {
        #[structopt(help = "Address")]
        address: String,
    },

    #[structopt(name = "printchain", about = "Print the chain")]
    PrintChain,

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
            Command::PrintChain => self.print_chain(),
            Command::Send { from, to, amount } => self.send(from.to_string(), to.to_string(), amount.clone()),
            Command::Getbalance { address } => self.get_balance(address.to_string()),
        }
    }

    fn create_blockchain(&self, address: String) {
        Blockchain::new(address.as_str());
        println!("Done");
    }

    fn print_chain(&self) {
        let bc = Blockchain::new("");
        let mut bci = bc.iterator();

        loop {
            if let Some(block) = bci.next() {
                println!("Prev. hash: {:}", hex::encode(&block.prev_block_hash));
                println!("Hash: {:}", hex::encode(&block.hash));
                let pow = ProofOfWork::new(&block);
                println!("PoW: {:}", pow.validate());
                println!("Transactions:");
                for (i, tx) in block.transactions.iter().enumerate() {
                    println!("tx {:}: {:}", i, tx);
                }
                println!();
            } else {
                break;
            }
        }
    }

    fn send(&self, from: String, to: String, amount: i64) {
        let mut bc = Blockchain::new("");
        let tx  = Transaction::new_utxo_transaction(from.to_string(), to.to_string(), amount, &bc);
        bc.mine_block(vec![tx]);
        println!("Success!");
    }

    fn get_balance(&self, address: String) {
        let bc = Blockchain::new("");
        let mut balance = 0;
        let utxos = bc.find_utxo(address.as_str());
        for out in utxos {
            balance += out.value;
        }
        println!("Balance of '{}': {}", address, balance);
    }
}