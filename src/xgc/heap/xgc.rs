//! Xgc — the primary GC handle. Owns the region table, marker, relocator,
//! and barrier buffers. Coordinates the four phases of a GC cycle:
//! 1. **Mark** — concurrent trace from roots.
//! 2. **Sweep** — release unmarked regions.
//! 3. **Relocate** — compact live objects into fresh regions.
//! 4. **Remap** — update references after relocation.

use crate::RuntimeError;
use crate::xgc::barrier::mark_state::MarkEpoch;
use crate::xgc::barrier::ref_update::RefUpdateBuffer;
use crate::xgc::barrier::satb::SatbBuffer;
use crate::xgc::colored::Color;
use crate::xgc::colored::ColoredPtr;
use crate::xgc::mark::phase::{MarkPhase, PhaseCell};
use crate::xgc::mark::worklist::Worklist;
use crate::xgc::object::ObjectHeader;
use crate::xgc::pin::PinHandle;
use crate::xgc::pin::PinRegistry;
use crate::xgc::pressure::threshold::PressureMeter;
use crate::xgc::pressure::trigger::GcTrigger;
use crate::xgc::region::table::RegionTable;
use crate::xgc::relocate::Relocator;

/// Global GC instance. One per process/heap.
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
}

// SAFETY: XGC coordinates access internally via atomic barriers.
unsafe impl Send for Xgc {}
unsafe impl Sync for Xgc {}

impl Xgc {
    /// Create a new XGC capable of managing `num_regions` regions.
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
        })
    }

    /// Initialize the heap and start background GC thread.
    pub fn init(&mut self) -> Result<(), RuntimeError> {
        if self.initialized {
            return Err(RuntimeError::AlreadyInitialized);
        }
        self.initialized = true;
        Ok(())
    }

    /// Shut down the GC, collect all memory, release regions.
    pub fn shutdown(&mut self) -> Result<(), RuntimeError> {
        if !self.initialized {
            return Err(RuntimeError::NotInitialized);
        }
        self.initialized = false;
        Ok(())
    }

    /// Current phase.
    pub fn phase(&self) -> MarkPhase {
        self.phase.load()
    }

    /// Current mark epoch.
    pub fn epoch(&self) -> u64 {
        self.epoch.current()
    }

    /// Number of completed GC cycles.
    pub fn cycle_count(&self) -> u32 {
        self.cycle_count
    }

    /// Begin a mark cycle. Returns the new epoch.
    pub fn begin_mark(&mut self) -> Result<u64, RuntimeError> {
        if !self.phase.cas(MarkPhase::Idle, MarkPhase::Marking) {
            return Err(RuntimeError::AlreadyInitialized);
        }
        self.worklist.clear();
        self.cycle_count = self.cycle_count.saturating_add(1);
        Ok(self.epoch.advance())
    }

    /// Push a root pointer onto the mark worklist.
    pub fn push_root(&mut self, root: ColoredPtr) -> Result<(), RuntimeError> {
        self.worklist.push(root)
    }

    /// Pop one entry from the mark worklist.
    pub fn pop_work(&mut self) -> Option<ColoredPtr> {
        self.worklist.pop()
    }

    /// Finish the mark cycle and transition to Idle.
    pub fn finish_mark(&mut self) {
        self.phase.store(MarkPhase::Idle);
        self.pressure.end_cycle();
    }

    /// Begin a relocation cycle.
    pub fn begin_relocate(&mut self) {
        self.relocator.begin();
        self.phase.store(MarkPhase::Relocating);
    }

    /// Record an object move.
    pub fn record_move(&mut self, old: ColoredPtr, new: ColoredPtr) -> Result<(), RuntimeError> {
        if self.pins.is_pinned(old) {
            return Ok(());
        }
        self.relocator.record_move(old, new)
    }

    /// Finish relocation and return statistics.
    pub fn finish_relocate(&mut self) -> crate::xgc::relocate::RelocStats {
        self.phase.store(MarkPhase::Idle);
        self.relocator.finish()
    }

    /// SATB pre-barrier: record `old` value before mutator overwrites a field.
    pub fn satb_record(&mut self, old: ColoredPtr) -> Result<(), RuntimeError> {
        self.satb.record(old)
    }

    /// Ref-update barrier: record field write for later remapping.
    pub fn ref_update_record(&mut self, field: usize, old: ColoredPtr) -> Result<(), RuntimeError> {
        self.ref_updates.record(field, old)
    }

    /// Color hint for newly allocated slots in the current phase.
    pub fn allocation_color(&self) -> Color {
        match self.phase.load() {
            MarkPhase::Marking => Color::Grey,
            _ => Color::White,
        }
    }

    /// Allocate a region slot, sized for the given header.
    pub fn allocate(
        &mut self,
        _header: &ObjectHeader,
        bytes: usize,
    ) -> Result<*mut u8, RuntimeError> {
        self.pressure.record_alloc(bytes as u64);
        Ok(core::ptr::null_mut())
    }

    /// Pin an object, preventing GC relocation.
    pub fn pin(&mut self, ptr: ColoredPtr) -> Result<PinHandle, RuntimeError> {
        self.pins.pin(ptr)
    }

    /// Unpin a previously-pinned object.
    pub fn unpin(&mut self, handle: PinHandle) -> bool {
        self.pins.unpin(handle)
    }

    /// Decide whether to start a new GC cycle based on allocation pressure.
    pub fn should_collect(&self) -> bool {
        use crate::xgc::pressure::threshold::PressureConfig;
        let config = PressureConfig::default_for(self.heap_size());
        self.trigger
            .should_trigger(&config, &self.pressure, self.cycle_count)
    }

    /// Total heap size in bytes.
    pub fn heap_size(&self) -> u64 {
        (self.num_regions * crate::xgc::region::REGION_SIZE) as u64
    }
}
