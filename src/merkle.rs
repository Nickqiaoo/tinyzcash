use bridgetree::BridgeTree;
use lazy_static::lazy_static;
use orchard::tree::MerkleHashOrchard;

lazy_static! {
    pub static ref MERKLE: BridgeTree::<MerkleHashOrchard, u32, 32> =
        BridgeTree::<MerkleHashOrchard, u32, 32>::new(100);
}
