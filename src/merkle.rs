use bridgetree::BridgeTree;
use orchard::tree::{MerkleHashOrchard, MerklePath};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref  MERKLE:BridgeTree::<MerkleHashOrchard, u32, 32> = BridgeTree::<MerkleHashOrchard, u32, 32>::new(100);
}
