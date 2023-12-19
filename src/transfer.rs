use orchard::{
    builder::Builder,
    bundle::{Authorized, Flags},
    circuit::{ProvingKey, VerifyingKey},
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope, SpendAuthorizingKey, SpendingKey},
    note::ExtractedNoteCommitment,
    note_encryption::OrchardDomain,
    tree::{MerkleHashOrchard, MerklePath},
    value::NoteValue,
    Bundle,
};
use crate::{wallets::Wallets};
use rand::rngs::OsRng;

pub fn transfer(address: String, value: u64) -> Bundle<Authorized, i64> {
    let wallets = Wallets::new();
    let wallet = wallets.get_wallet(&address).unwrap();
    
    let mut rng = OsRng;
    let pk = ProvingKey::build();

    let sk = wallet.sk();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    let shielded_bundle: Bundle<_, i64> = {
        let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
        let (note, _, _) = shielding_bundle
            .actions()
            .iter()
            .find_map(|action| {
                let domain = OrchardDomain::for_action(action);
                try_note_decryption(&domain, &ivk, action)
            })
            .unwrap();

        // Use the tree with a single leaf.
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
        assert_eq!(builder.add_spend(fvk, note, merkle_path), Ok(()));
        assert_eq!(
            builder.add_recipient(None, recipient, NoteValue::from_raw(5000), None),
            Ok(())
        );
        let unauthorized = builder.build(&mut rng).unwrap();
        let sighash = unauthorized.commitment().into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        proven
            .apply_signatures(rng, sighash, &[SpendAuthorizingKey::from(&sk)])
            .unwrap()
    };
    shielded_bundle
}