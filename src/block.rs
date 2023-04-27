struct Block {
    timestamp: i64,
    data: &[u8],
    prev_block_hash: &[u8],
    hash: &[u8],
}
