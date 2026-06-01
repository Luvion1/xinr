//! Weak references: pointers that do not prevent GC of the referent.
//!
//! A `WeakRef` is upgraded to a `Strong` only if the object is still alive.
//! After upgrade fails, the weak reference is dead and cannot be reused.

use crate::RuntimeError;
use crate::xgc::colored::{Color, ColoredPtr};

pub const WEAK_TABLE_CAP: usize = 256;

/// One weak reference entry.
#[derive(Debug, Clone, Copy)]
pub struct WeakEntry {
    pub id: u32,
    pub target: ColoredPtr,
    pub alive: bool,
}

/// Opaque weak reference handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WeakRef(pub u32);

/// Weak reference table.
pub struct WeakTable {
    entries: [WeakEntry; WEAK_TABLE_CAP],
    next_id: u32,
}

impl WeakTable {
    /// Construct an empty weak table.
    pub const fn new() -> Self {
        Self {
            entries: [WeakEntry {
                id: 0,
                target: ColoredPtr::new(0, Color::White),
                alive: false,
            }; WEAK_TABLE_CAP],
            next_id: 1,
        }
    }

    /// Create a weak reference to `target`.
    pub fn create(&mut self, target: ColoredPtr) -> Option<WeakRef> {
        for e in self.entries.iter_mut() {
            if !e.alive {
                let id = self.next_id;
                self.next_id = self.next_id.wrapping_add(1);
                e.id = id;
                e.target = target;
                e.alive = true;
                return Some(WeakRef(id));
            }
        }
        None
    }

    /// Try to upgrade a weak reference. Returns the target if still alive.
    pub fn upgrade(&self, w: WeakRef) -> Option<ColoredPtr> {
        for e in self.entries.iter() {
            if e.alive && e.id == w.0 {
                return Some(e.target);
            }
        }
        None
    }

    /// Mark a weak reference as dead (called during GC sweep).
    pub fn invalidate(&mut self, target: ColoredPtr) -> usize {
        let mut n = 0;
        for e in self.entries.iter_mut() {
            if e.alive && e.target == target {
                e.alive = false;
                n += 1;
            }
        }
        n
    }

    /// Number of currently live weak references.
    pub fn len(&self) -> usize {
        self.entries.iter().filter(|e| e.alive).count()
    }

    /// Whether the table has no live weak references.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Check existence and return whether target matches.
    pub fn contains(&self, w: WeakRef) -> bool {
        self.entries.iter().any(|e| e.alive && e.id == w.0)
    }
}

impl Default for WeakTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Attempt to upgrade a weak reference. Errors if the ref is dead.
pub fn try_upgrade(table: &WeakTable, w: WeakRef) -> Result<ColoredPtr, RuntimeError> {
    table.upgrade(w).ok_or(RuntimeError::NoParkingPermit)
}
