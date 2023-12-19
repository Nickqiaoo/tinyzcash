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
mod deposit;
mod merkle;
mod transfer;

fn main() {
    let mut c = cli::Cli {
        cmd: cli::Command::from_args(),
    };
    c.run();
}
