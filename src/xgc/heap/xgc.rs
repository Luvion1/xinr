//! Xgc — the primary GC handle.

use crate::RuntimeError;
use crate::xgc::barrier::mark_state::MarkEpoch;
use crate::xgc::barrier::ref_update::RefUpdateBuffer;
use crate::xgc::barrier::satb::SatbBuffer;
use crate::xgc::colored::Color;
use crate::xgc::colored::ColoredPtr;
use crate::xgc::mark::phase::{MarkPhase, PhaseCell};
use crate::xgc::mark::worklist::Worklist;
use crate::xgc::object::ObjectHeader;
use crate::xgc::object::layout::Alignment;
use crate::xgc::pin::PinHandle;
use crate::xgc::pin::PinRegistry;
use crate::xgc::pressure::threshold::PressureMeter;
use crate::xgc::pressure::trigger::GcTrigger;
use crate::xgc::region::table::RegionTable;
use crate::xgc::relocate::Relocator;
use core::alloc::Layout;
use core::mem;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

unsafe impl Send for Xgc {}
unsafe impl Sync for Xgc {}

/// Global GC handle.
pub struct Xgc {
    num_regions: usize,
    initialized: bool,
    phase: PhaseCell,
    epoch: MarkEpoch,
    worklist: Worklist,
    satb: SatbBuffer,
    ref_updates: RefUpdateBuffer,
    relocator: Relocator,
    pressure: PressureMeter,
    trigger: GcTrigger,
    pins: PinRegistry,
    cycle_count: u32,
    #[cfg(feature = "alloc")]
    live_objects: Vec<LiveObject>,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy)]
struct LiveObject {
    base: *mut u8,
    total_bytes: usize,
    color: Color,
}

impl Xgc {
    pub fn new(num_regions: usize) -> Result<Self, RuntimeError> {
        if num_regions == 0 {
            return Err(RuntimeError::OutOfMemory);
        }
        let _ = RegionTable::new(num_regions)?;
        Ok(Self {
            num_regions,
            initialized: false,
            phase: PhaseCell::new(),
            epoch: MarkEpoch::new(),
            worklist: Worklist::new(),
            satb: SatbBuffer::new(),
            ref_updates: RefUpdateBuffer::new(),
            relocator: Relocator::new(),
            pressure: PressureMeter::new(),
            trigger: GcTrigger::new(),
            pins: PinRegistry::new(),
            cycle_count: 0,
            #[cfg(feature = "alloc")]
            live_objects: Vec::new(),
        })
    }

    pub fn init(&mut self) -> Result<(), RuntimeError> {
        if self.initialized {
            return Err(RuntimeError::AlreadyInitialized);
        }
        self.initialized = true;
        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<(), RuntimeError> {
        if !self.initialized {
            return Err(RuntimeError::NotInitialized);
        }
        self.initialized = false;
        Ok(())
    }

    pub fn phase(&self) -> MarkPhase {
        self.phase.load()
    }

    pub fn epoch(&self) -> u64 {
        self.epoch.current()
    }

    pub fn cycle_count(&self) -> u32 {
        self.cycle_count
    }

    pub fn begin_mark(&mut self) -> Result<u64, RuntimeError> {
        if !self.phase.cas(MarkPhase::Idle, MarkPhase::Marking) {
            return Err(RuntimeError::AlreadyMarking);
        }
        self.worklist.clear();
        self.cycle_count = self.cycle_count.saturating_add(1);
        Ok(self.epoch.advance())
    }

    pub fn push_root(&mut self, root: ColoredPtr) -> Result<(), RuntimeError> {
        self.worklist.push(root)
    }

    pub fn pop_work(&mut self) -> Option<ColoredPtr> {
        self.worklist.pop()
    }

    pub fn finish_mark(&mut self) {
        self.phase.store(MarkPhase::Idle);
        self.pressure.end_cycle();
    }

    pub fn begin_relocate(&mut self) {
        self.relocator.begin();
        self.phase.store(MarkPhase::Relocating);
    }

    pub fn record_move(&mut self, old: ColoredPtr, new: ColoredPtr) -> Result<(), RuntimeError> {
        if self.pins.is_pinned(old) {
            return Ok(());
        }
        self.relocator.record_move(old, new)
    }

    pub fn finish_relocate(&mut self) -> crate::xgc::relocate::RelocStats {
        self.phase.store(MarkPhase::Idle);
        self.relocator.finish()
    }

    pub fn satb_record(&mut self, old: ColoredPtr) -> Result<(), RuntimeError> {
        self.satb.record(old)
    }

    pub fn ref_update_record(&mut self, field: usize, old: ColoredPtr) -> Result<(), RuntimeError> {
        self.ref_updates.record(field, old)
    }

    pub fn allocation_color(&self) -> Color {
        match self.phase.load() {
            MarkPhase::Marking => Color::Grey,
            _ => Color::White,
        }
    }

    /// Allocate memory backed by the system allocator.
    /// Writes ObjectHeader and returns user payload pointer.
    pub fn allocate(
        &mut self,
        header: &ObjectHeader,
        bytes: usize,
    ) -> Result<*mut u8, RuntimeError> {
        self.pressure.record_alloc(bytes as u64);

        let mut total = bytes.saturating_add(mem::size_of::<ObjectHeader>());
        let align = Alignment::A64.bytes();
        total = (total + align - 1) & !(align - 1);

        let layout = Layout::from_size_align(total, align)
            .map_err(|_| RuntimeError::OutOfMemory)?;

        #[cfg(feature = "alloc")]
        {
            use alloc::alloc;
            let base = unsafe { alloc::alloc(layout) };
            if base.is_null() {
                return Err(RuntimeError::OutOfMemory);
            }
            unsafe {
                let hdr_slot = base as *mut ObjectHeader;
                hdr_slot.write(*header);
            }
            self.live_objects.push(LiveObject {
                base,
                total_bytes: total,
                color: Color::White,
            });
            Ok(unsafe { base.add(mem::size_of::<ObjectHeader>()) })
        }

        #[cfg(not(feature = "alloc"))]
        {
            let _ = layout;
            Err(RuntimeError::OutOfMemory)
        }
    }

    /// Sweep: free unmarked (White) objects. Returns count freed and bytes reclaimed.
    pub fn sweep(&mut self) -> (u32, u64) {
        #[cfg(feature = "alloc")]
        {
            use alloc::alloc;
            let mut freed = 0u32;
            let mut reclaimed = 0u64;
            let mut remaining = Vec::new();

            for obj in self.live_objects.drain(..) {
                let hdr = unsafe { obj.base as *const ObjectHeader };
                let color = unsafe { (*hdr).color() };
                if color == Color::White && !self.pins.is_pinned(ColoredPtr::new(obj.base as usize, Color::White)) {
                    let layout = Layout::from_size_align(obj.total_bytes, Alignment::A64.bytes()).unwrap();
                    unsafe { alloc::dealloc(obj.base, layout) };
                    freed += 1;
                    reclaimed += obj.total_bytes as u64;
                    self.pressure.record_free(obj.total_bytes as u64);
                } else {
                    remaining.push(obj);
                }
            }
            self.live_objects = remaining;
            (freed, reclaimed)
        }

        #[cfg(not(feature = "alloc"))]
        (0, 0)
    }

    /// Scan all live objects and apply a visitor to outgoing pointers.
    pub fn scan_objects<F>(&mut self, mut visitor: F)
    where
        F: FnMut(*mut u8),
    {
        #[cfg(feature = "alloc")]
        {
            for obj in &self.live_objects {
                visitor(obj.base);
            }
        }
    }

    pub fn pin(&mut self, ptr: ColoredPtr) -> Result<PinHandle, RuntimeError> {
        self.pins.pin(ptr)
    }

    pub fn unpin(&mut self, handle: PinHandle) -> bool {
        self.pins.unpin(handle)
    }

    pub fn should_collect(&self) -> bool {
        use crate::xgc::pressure::threshold::PressureConfig;
        let config = PressureConfig::default_for(
            (self.num_regions * crate::xgc::region::REGION_SIZE) as u64,
        );
        self.trigger
            .should_trigger(&config, &self.pressure, self.cycle_count)
    }

    pub fn forward(&self, ptr: ColoredPtr) -> ColoredPtr {
        self.relocator.forward(ptr)
    }
}
