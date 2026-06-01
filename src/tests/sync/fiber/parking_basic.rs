//! Parking lot tests.

use crate::sync::parking::lot::ParkingLot;
use crate::sync::parking::permit::Permit;

// --- Permit ---

#[test]
fn permit_none_is_invalid() {
    assert!(!Permit::NONE.is_valid());
}

#[test]
fn permit_valid_after_acquire() {
    let mut lot = ParkingLot::new();
    let p = lot.acquire_permit();
    assert!(p.is_valid());
}

#[test]
fn permit_tokens_are_unique() {
    let mut lot = ParkingLot::new();
    let p1 = lot.acquire_permit();
    let p2 = lot.acquire_permit();
    assert_ne!(p1, p2);
}

// --- ParkingLot ---

#[test]
fn parking_lot_empty() {
    let lot = ParkingLot::new();
    assert_eq!(lot.parked_count(), 0);
    assert_eq!(lot.capacity(), 64);
}

#[test]
fn parking_lot_park_unpark() {
    let mut lot = ParkingLot::new();
    let permit = lot.acquire_permit();
    let token = lot.park(1, permit, 100).unwrap();
    assert_eq!(lot.parked_count(), 1);
    let rec = lot.unpark(token).unwrap();
    assert_eq!(rec.thread_id, 1);
    assert_eq!(rec.parked_at, 100);
    assert_eq!(lot.parked_count(), 0);
}

#[test]
fn parking_lot_unpark_by_thread() {
    let mut lot = ParkingLot::new();
    let permit = lot.acquire_permit();
    lot.park(7, permit, 200).unwrap();
    let rec = lot.unpark_thread(7).unwrap();
    assert_eq!(rec.thread_id, 7);
    assert_eq!(rec.parked_at, 200);
}

#[test]
fn parking_lot_peek_doesnt_remove() {
    let mut lot = ParkingLot::new();
    let permit = lot.acquire_permit();
    let token = lot.park(9, permit, 50).unwrap();
    assert!(lot.peek(token).is_some());
    assert_eq!(lot.parked_count(), 1);
}

#[test]
fn parking_lot_double_park_same_thread() {
    let mut lot = ParkingLot::new();
    let p1 = lot.acquire_permit();
    let p2 = lot.acquire_permit();
    lot.park(3, p1, 0).unwrap();
    lot.park(3, p2, 0).unwrap();
    assert_eq!(lot.parked_count(), 2);
    lot.unpark_thread(3).unwrap();
    assert_eq!(lot.parked_count(), 1);
}

#[test]
fn parking_lot_unpark_unknown() {
    let mut lot = ParkingLot::new();
    assert!(lot.unpark(999).is_none());
    assert!(lot.unpark_thread(999).is_none());
}

#[test]
fn parking_lot_active_tokens() {
    let mut lot = ParkingLot::new();
    let p1 = lot.acquire_permit();
    let p2 = lot.acquire_permit();
    let t1 = lot.park(1, p1, 0).unwrap();
    let t2 = lot.park(2, p2, 0).unwrap();
    let tokens = lot.active_tokens();
    assert!(tokens.contains(&Some(t1)));
    assert!(tokens.contains(&Some(t2)));
}
