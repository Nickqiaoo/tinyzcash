use structopt::StructOpt;

mod block;
mod blockchain;
mod pow;
mod cli;
mod iterator;
mod transaction;
mod transaction_output;
mod transaction_input;
mod wallet;

fn main() {
    let mut c = cli::CLI{
        cmd:cli::Command::from_args()
    };
    c.run()
}