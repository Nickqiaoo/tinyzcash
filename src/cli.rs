use structopt::StructOpt;

use crate::blockchain;

pub struct CLI {
    pub bc: blockchain::Blockchain,
    pub cmd: Command
}

#[derive(StructOpt, Debug)]
#[structopt(name = "blockchain", about = "A simple CLI application")]
pub enum Command {
    #[structopt(name = "addblock", about = "Add a block to the chain")]
    AddBlock {
        #[structopt(help = "Block data")]
        data: String,
    },
    #[structopt(name = "printchain", about = "Print the chain")]
    PrintChain,
}


impl CLI {
    pub fn run(&mut self) {
        let data = if let Command::AddBlock { data } = &self.cmd {
            Some(data.clone())
        } else {
            None
        };

        match self.cmd {
            Command::AddBlock { data: _ } => self.add_block(data.unwrap()),
            Command::PrintChain => self.print_chain(),
        }
    }

    fn add_block(&mut self, data: String) {
        // Add block implementation
        self.bc.add_block(data.as_str())
    }

    fn print_chain(&self) {
        // Print chain implementation
    }
}