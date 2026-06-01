//! GC event: a single timeline entry.

/// GC event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Init,
    MarkStart,
    MarkEnd,
    RelocateStart,
    RelocateEnd,
    SweepStart,
    SweepEnd,
    Alloc,
    Promote,
    Evict,
    Cycle,
    Trigger,
    Error,
}

impl EventKind {
    /// Short string label for the event.
    pub fn label(self) -> &'static str {
        match self {
            EventKind::Init => "init",
            EventKind::MarkStart => "mark.start",
            EventKind::MarkEnd => "mark.end",
            EventKind::RelocateStart => "reloc.start",
            EventKind::RelocateEnd => "reloc.end",
            EventKind::SweepStart => "sweep.start",
            EventKind::SweepEnd => "sweep.end",
            EventKind::Alloc => "alloc",
            EventKind::Promote => "promote",
            EventKind::Evict => "evict",
            EventKind::Cycle => "cycle",
            EventKind::Trigger => "trigger",
            EventKind::Error => "error",
        }
    }
}

/// Single GC event.
#[derive(Debug, Clone, Copy)]
pub struct GcEvent {
    pub kind: EventKind,
    pub t_ms: u64,
    pub value: u64,
}

impl GcEvent {
    /// Construct a new event.
    pub const fn new(kind: EventKind, t_ms: u64, value: u64) -> Self {
        Self { kind, t_ms, value }
    }
}
