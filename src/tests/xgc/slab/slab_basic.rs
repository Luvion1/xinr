//! Slab allocator tests.

#[cfg(test)]
mod tests {
    use crate::xgc::slab::SlabAllocator;

    #[test]
    fn slab_allocator_reserves_memory() {
        let slab = SlabAllocator::new(0x10000);
        assert_eq!(slab.reserved(), 0);
        assert_eq!(slab.slab_count(), 0);
    }

    #[test]
    fn slab_alloc_grows_slab_count() {
        let mut slab = SlabAllocator::new(0x10000);
        let ptr = slab.alloc(48);
        assert!(ptr.is_some());
        assert_eq!(slab.slab_count(), 1);
    }

    #[test]
    fn slab_alloc_returns_different_addresses() {
        let mut slab = SlabAllocator::new(0x10000);
        let a = slab.alloc(48).unwrap();
        let b = slab.alloc(48).unwrap();
        assert_ne!(a.addr(), b.addr());
    }
}