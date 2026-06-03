//! XGC main handle tests.

use crate::xgc::Xgc;

#[test]
fn xgc_new_succeeds() {
    let gc = Xgc::new(8);
    assert!(gc.is_ok());
}

#[test]
fn xgc_new_fails_zero() {
    let gc = Xgc::new(0);
    assert!(gc.is_err());
}

#[test]
fn xgc_init_transition() {
    let mut gc = Xgc::new(4).unwrap();
    assert!(gc.init().is_ok());
    assert!(gc.shutdown().is_ok());
}
