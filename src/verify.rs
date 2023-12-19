use orchard::{
    bundle::Authorized,
    circuit::VerifyingKey,
    Bundle,
};

pub fn verify_bundle(bundle: &Bundle<Authorized, i64>) {
    let vk = VerifyingKey::build();
    assert!(matches!(bundle.verify_proof(&vk), Ok(())));
    let sighash: [u8; 32] = bundle.commitment().into();
    let bvk = bundle.binding_validating_key();
    for action in bundle.actions() {
        assert_eq!(action.rk().verify(&sighash, action.authorization()), Ok(()));
    }
    assert_eq!(
        bvk.verify(&sighash, bundle.authorization().binding_signature()),
        Ok(())
    );
}