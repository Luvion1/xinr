//! Oneshot tests.

use crate::RuntimeError;
use crate::sync::oneshot::Oneshot;

#[test]
fn oneshot_send_recv() {
    let mut o: Oneshot<u32> = Oneshot::new();
    assert!(!o.is_ready());
    o.send(42).unwrap();
    assert!(o.is_ready());
    assert_eq!(o.recv().unwrap(), 42);
}

#[test]
fn oneshot_double_send_rejected() {
    let mut o: Oneshot<u32> = Oneshot::new();
    o.send(1).unwrap();
    assert_eq!(o.send(2), Err(RuntimeError::WouldBlock));
}

#[test]
fn oneshot_recv_empty_rejected() {
    let mut o: Oneshot<u32> = Oneshot::new();
    assert_eq!(o.recv(), Err(RuntimeError::WouldBlock));
}
