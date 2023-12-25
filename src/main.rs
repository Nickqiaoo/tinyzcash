use structopt::StructOpt;

mod block;
mod blockchain;
mod cli;
mod deposit;
mod iterator;
mod merkle;
mod pow;
mod transaction;
mod transaction_input;
mod transaction_output;
mod verify;
mod wallet;
mod wallets;
mod withdraw;
mod zsend;
mod bundle;

fn main() {
    let mut c = cli::Cli {
        cmd: cli::Command::from_args(),
    };
    c.run();
}
