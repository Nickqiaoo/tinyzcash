mod block;
mod blockchain;
mod pow;

use hex::encode;
fn main() {
    let mut bc = blockchain::Blockchain::new();

    bc.add_block("Send 1 BTC to Ivan");
    bc.add_block("Send 2 more BTC to Ivan");

    for block in &bc.blocks {
        println!("Prev. hash: {}", encode(&block.prev_block_hash));
        println!("Data: {}", std::str::from_utf8(&block.data).unwrap());
        println!("Hash: {}\n", encode(&block.hash));
    }
}