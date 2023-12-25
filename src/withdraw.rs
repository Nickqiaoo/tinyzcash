use crate::wallets::Wallets;
use bridgetree::BridgeTree;
use orchard::builder::Builder;
use orchard::bundle::{Authorized, Flags};
use orchard::circuit::ProvingKey;
use orchard::keys::{FullViewingKey, SpendAuthorizingKey};
use orchard::note::ExtractedNoteCommitment;
use orchard::tree::{MerkleHashOrchard, MerklePath};
use orchard::Bundle;
use rand::rngs::OsRng;

pub fn withdraw(address: &str) -> Bundle<Authorized, i64> {
    let wallets = Wallets::new();
    let wallet = wallets.get_z_wallet(address).unwrap();

    let mut rng = OsRng;
    let pk = ProvingKey::build();

    let sk = wallet.sk();
    let fvk = FullViewingKey::from(&sk);

    // Create a shielding bundle.
    let shielding_bundle: Bundle<_, i64> = {
        let old_note = wallet.notes.get(0).unwrap();
        let note = old_note.to_note(wallet.z_address());
        let cmx: ExtractedNoteCommitment = note.commitment().into();

        let leaf = MerkleHashOrchard::from_cmx(&cmx);
        let mut tree = BridgeTree::<MerkleHashOrchard, u32, 32>::new(100);
        tree.append(leaf);
        let position = tree.mark().unwrap();
        let root = tree.root(0).unwrap();
        let auth_path = tree.witness(position, 0).unwrap();
        let merkle_path = MerklePath::from_parts(
            u64::from(position).try_into().unwrap(),
            auth_path[..].try_into().unwrap(),
        );
        let anchor = root.into();
        assert_eq!(anchor, merkle_path.root(cmx));

        let mut builder = Builder::new(Flags::from_parts(true, false), anchor);
        assert_eq!(builder.add_spend(fvk, note, merkle_path), Ok(()));
        let unauthorized = builder.build(&mut rng).unwrap();
        let sighash = unauthorized.commitment().into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };
    shielding_bundle
}

pub fn save_note(address: &str) {
    let mut wallets = Wallets::new();
    let wallet = wallets.get_mut_z_wallet(address);
    wallet.notes.remove(0);
    _ = wallets.save_to_file();
}
