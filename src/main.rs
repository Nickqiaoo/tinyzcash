use structopt::StructOpt;

mod block;
mod blockchain;
mod cli;
mod iterator;
mod pow;
mod transaction;
mod transaction_input;
mod transaction_output;
mod wallet;
mod wallets;

fn main() {
    let mut c = cli::CLI {
        cmd: cli::Command::from_args(),
    };
    c.run();
}
