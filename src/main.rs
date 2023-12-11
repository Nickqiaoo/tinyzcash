use structopt::StructOpt;

mod block;
mod blockchain;
mod circuit;
mod cli;
mod coin;
mod iterator;
mod mint;
mod pour;
mod pow;
mod transaction;
mod transaction_input;
mod transaction_output;
mod wallet;
mod wallets;

fn main() {
    let mut c = cli::Cli {
        cmd: cli::Command::from_args(),
    };
    c.run();
}
