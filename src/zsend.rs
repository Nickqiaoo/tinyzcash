use crate::{wallet, wallets::Wallets};
use bridgetree::BridgeTree;
use orchard::{
    builder::Builder,
    bundle::{Authorized, Flags},
    circuit::ProvingKey,
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendAuthorizingKey},
    note::ExtractedNoteCommitment,
    note_encryption::OrchardDomain,
    tree::{MerkleHashOrchard, MerklePath},
    value::NoteValue,
    Bundle,
};
use rand::rngs::OsRng;
use zcash_note_encryption::try_note_decryption;

pub fn zsend(from: &str, to: &str) -> Bundle<Authorized, i64> {
    let wallets = Wallets::new();

    let mut rng = OsRng;
    let pk = ProvingKey::build();

    let from = wallets.get_z_wallet(from).unwrap();
    let from_sk = from.sk();
    let from_fvk = FullViewingKey::from(&from_sk);
    let from_addr = from.z_address();

    let to = wallets.get_z_wallet(to).unwrap();
    let recipient = to.z_address();

    let shielded_bundle: Bundle<_, i64> = {
        let old_note = from.notes.get(0).unwrap();
        let note = old_note.to_note(from_addr);
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

        let mut builder = Builder::new(Flags::from_parts(true, true), anchor);
        assert_eq!(builder.add_spend(from_fvk, note, merkle_path), Ok(()));
        assert_eq!(
            builder.add_recipient(None, recipient, NoteValue::from_raw(old_note.value), None),
            Ok(())
        );
        let unauthorized = builder.build(&mut rng).unwrap();
        let sighash = unauthorized.commitment().into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&from_sk)])
            .unwrap()
    };
    shielded_bundle
}

pub fn save_note(bundle: &Bundle<Authorized, i64>, from: &str, to: &str) {
    let mut wallets = Wallets::new();
    let to_wallet = wallets.get_z_wallet(to).unwrap();
    let to_sk = to_wallet.sk();
    let to_fvk = FullViewingKey::from(&to_sk);
    let to_ivk = PreparedIncomingViewingKey::new(&to_fvk.to_ivk(Scope::External));

    let (note, _, _) = bundle
        .actions()
        .iter()
        .find_map(|action| {
            let domain = OrchardDomain::for_action(action);
            try_note_decryption(&domain, &to_ivk, action)
        })
        .unwrap();
    let n = wallet::Note {
        value: note.value().inner(),
        rseed: *note.rseed().as_bytes(),
        nf: note.rho().to_bytes(),
    };

    let wallet = wallets.get_mut_z_wallet(to);
    wallet.notes.push(n);

    let wallet = wallets.get_mut_z_wallet(from);
    wallet.notes.remove(0);
    _ = wallets.save_to_file();
}
