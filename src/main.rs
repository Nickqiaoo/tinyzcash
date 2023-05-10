use structopt::StructOpt;

mod block;
mod blockchain;
mod pow;
mod cli;

fn main() {
    let bc = blockchain::Blockchain::new();
    let mut c = cli::CLI{
        bc,
        cmd:cli::Command::from_args()
    };
    c.run()
}