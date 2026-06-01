//! XGC — Xin Garbage Collector.
//!
//! Region-based, concurrent, ZGC-inspired GC.
//!
//! # Architecture Overview
//!
//! - Heap is divided into fixed-size regions (1 MiB).
//! - Concurrent marking + compaction with load barriers.
//! - Colored pointers track object liveness with minimal overhead.
//! - `no_std` compatible; `alloc` feature required for heap structures.
//!
//! # Example
//!
//! ```ignore
//! use xinr::xgc::Xgc;
//! let mut gc = Xgc::new(1024)?;
//! gc.init()?;
//! gc.begin_mark()?;
//! // ... push roots, trace, etc.
//! gc.finish_mark();
//! ```

#![cfg(feature = "alloc")]

pub mod barrier;
pub mod budget;
pub mod card;
pub mod colored;
pub mod cycle;
pub mod diagnostics;
pub mod finalize;
pub mod hazard;
pub mod heap;
pub mod heuristics;
pub mod log;
pub mod mark;
pub mod numa;
pub mod object;
pub mod page;
pub mod pin;
pub mod pressure;
pub mod profile;
pub mod region;
pub mod relocate;
pub mod sched;
pub mod slab;
pub mod tl;
pub mod worker;

pub use barrier::load::LoadBarrier;
pub use barrier::mark_state::MarkEpoch;
pub use barrier::ref_update::RefUpdateBuffer;
pub use barrier::satb::SatbBuffer;
pub use budget::clock::{Instant, duration_ms};
pub use budget::{BudgetTracker, CycleBudget};
pub use card::byte::{CARD_SIZE, CARDS_PER_REGION, CardByte, CardState};
pub use card::table::CardTable;
pub use colored::{Color, ColoredPtr};
pub use cycle::CycleDetector;
pub use cycle::queue::CycleQueue;
pub use cycle::stats::{CycleCandidate, CycleStats};
pub use finalize::queue::{FinalizationQueue, FinalizeEntry, FinalizerId, QUEUE_CAP};
pub use finalize::weak::{WEAK_TABLE_CAP, WeakEntry, WeakRef, WeakTable, try_upgrade};
pub use hazard::record::HazardRecord;
pub use hazard::slot::HazardSlot;
pub use heap::Xgc;
pub use mark::phase::{MarkPhase, PhaseCell};
pub use mark::worklist::{WORKLIST_CAP, Worklist};
pub use numa::node::NodeId;
pub use numa::topology::{NodeCapacity, NumaTopology};
pub use object::header::{HEADER_MAGIC, ObjectHeader};
pub use object::layout::{Alignment, SizeClass};
pub use object::traits::{Trace, Visitor};
pub use page::align::{
    PAGE_SIZE, REGION_PAGE_COUNT, is_page_aligned, page_round_down, page_round_up,
};
pub use page::descriptor::{PageDescriptor, PageState};
pub use page::table::PageTable;
pub use pin::handle::{PinHandle, PinnedObject, handle_from_id};
pub use pin::registry::{PinEntry, PinRegistry};
pub use pressure::threshold::{PressureConfig, PressureMeter};
pub use pressure::trigger::GcTrigger;
pub use profile::site::{NEXT_SITE, SiteId};
pub use profile::stats::{SiteEntry, SiteStats};
pub use region::table::RegionTable;
pub use region::{REGION_SIZE, Region};
pub use relocate::{RelocStats, Relocator};
pub use sched::deque::WorkDeque;
pub use sched::pool::WorkerPool;
pub use slab::SlabAllocator;
pub use slab::size::SlabSize;
pub use worker::signal::{GcSignal, WorkOrder};
pub use worker::thread::{GcWorker, WorkerState};
