use std::println;

use structopt::StructOpt;
use crate::{blockchain::Blockchain, pow::ProofOfWork};

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
     
}

impl CLI {
    pub fn run(&mut self) {
        match &self.cmd {
            Command::CreateBlockChain { address } => self.create_blockchain(address.to_string()),
            Command::PrintChain => self.print_chain(),
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
                println!();
            } else {
                break;
            }
        }
    }
}