use crate::{merkle, wallet, wallets::Wallets};
use orchard::circuit::ProvingKey;
use orchard::{
    builder::Builder,
    bundle::{Authorized, Flags},
    keys::{FullViewingKey, PreparedIncomingViewingKey, Scope},
    note_encryption::OrchardDomain,
    value::NoteValue,
    Bundle,
};
use rand::rngs::OsRng;
use zcash_note_encryption::try_note_decryption;

pub fn deposit(address: &String, value: u64) -> Bundle<Authorized, i64> {
    let wallets = Wallets::new();
    let wallet = wallets.get_wallet(address).unwrap();

    let mut rng = OsRng;
    let pk = ProvingKey::build();

    let sk = wallet.sk();
    let fvk = FullViewingKey::from(&sk);
    let recipient = fvk.address_at(0u32, Scope::External);

    // Create a shielding bundle.
    let shielding_bundle: Bundle<_, i64> = {
        // Use the empty tree.
        let anchor = merkle::MERKLE.root(0).unwrap().into();

        let mut builder = Builder::new(Flags::from_parts(false, true), anchor);
        assert_eq!(
            builder.add_recipient(None, recipient, NoteValue::from_raw(value), None),
            Ok(())
        );
        let unauthorized = builder.build(&mut rng).unwrap();
        let sighash = unauthorized.commitment().into();
        let proven = unauthorized.create_proof(&pk, &mut rng).unwrap();
        proven.apply_signatures(rng, sighash, &[]).unwrap()
    };
    shielding_bundle
}

pub fn save_note(bundle: &Bundle<Authorized, i64>, address: &String) {
    let mut wallets = Wallets::new();
    let wallet = wallets.get_wallet(address).unwrap();
    let sk = wallet.sk();
    let fvk = FullViewingKey::from(&sk);
    let ivk = PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));

    let (note, _, _) = bundle
        .actions()
        .iter()
        .find_map(|action| {
            let domain = OrchardDomain::for_action(action);
            try_note_decryption(&domain, &ivk, action)
        })
        .unwrap();
    let n = wallet::Note {
        value: note.value().inner(),
        rseed: *note.rseed().as_bytes(),
        nf: note.rho().to_bytes(),
    };

    let wallet = wallets.get_mut_wallet(address);
    wallet.notes.push(n);
    _ = wallets.save_to_file();
}
