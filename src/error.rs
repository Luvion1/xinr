//! Runtime error types for `xinr`.
//!
//! Always compiles under `no_std` with no dependencies. Error strings are
//! either formatted from parameters or baked into the variant.

use core::fmt;

/// Result alias used throughout `xinr`.
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// Runtime-level errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuntimeError {
    #[default]
    Ok = 0,

    /// Failed to allocate a new memory region.
    OutOfMemory = 1,

    /// XGC is already initialized for this process.
    AlreadyInitialized = 2,

    /// XGC has not been initialized; call `Xgc::init()` first.
    NotInitialized = 3,

    /// Fiber stack overflow: limit exceeded.
    StackOverflow = 4,

    /// No parking permit available for the current thread.
    NoParkingPermit = 5,

    /// Invalid region index (out of bounds).
    InvalidRegion = 6,

    /// Channel/scope is disconnected; sender or receiver was dropped.
    Disconnected = 7,

    /// Operation would block (try_send/try_recv).
    WouldBlock = 8,

    /// Channel or scope is closed.
    Closed = 9,

    /// Operation timed out.
    TimedOut = 10,
}

impl RuntimeError {
    /// Numeric code for interop with host environment or macro-based matching.
    pub const fn code(self) -> usize {
        self as usize
    }

    /// Whether this error is transient (the operation can be retried).
    pub const fn is_transient(self) -> bool {
        matches!(self, Self::WouldBlock | Self::TimedOut)
    }

    /// Whether this error indicates a permanent failure.
    pub const fn is_fatal(self) -> bool {
        matches!(self, Self::OutOfMemory | Self::StackOverflow)
    }
}

impl From<()> for RuntimeError {
    fn from(_: ()) -> Self {
        Self::Ok
    }
}

impl From<core::convert::Infallible> for RuntimeError {
    fn from(_: core::convert::Infallible) -> Self {
        Self::Ok
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::Ok => "ok",
            Self::OutOfMemory => "out of memory",
            Self::AlreadyInitialized => "already initialized",
            Self::NotInitialized => "not initialized",
            Self::StackOverflow => "stack overflow",
            Self::NoParkingPermit => "no parking permit",
            Self::InvalidRegion => "invalid region",
            Self::Disconnected => "disconnected",
            Self::WouldBlock => "would block",
            Self::Closed => "closed",
            Self::TimedOut => "timed out",
        };
        write!(f, "{}", msg)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RuntimeError {}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use core::fmt::Write;

    fn render_len(e: RuntimeError) -> usize {
        let mut buf: [u8; 64] = [0; 64];
        let mut s = Scribe::new(&mut buf);
        let _ = write!(&mut s, "{}", e);
        s.len()
    }

    struct Scribe<'a> {
        buf: &'a mut [u8],
        pos: usize,
    }
    impl<'a> Scribe<'a> {
        fn new(buf: &'a mut [u8]) -> Self {
            Self { buf, pos: 0 }
        }
        fn len(&self) -> usize {
            self.pos
        }
    }
    impl<'a> core::fmt::Write for Scribe<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let room = self.buf.len().saturating_sub(self.pos);
            let n = bytes.len().min(room);
            self.buf[self.pos..self.pos + n].copy_from_slice(&bytes[..n]);
            self.pos += n;
            Ok(())
        }
    }

    #[test]
    fn code_matches_variant_index() {
        assert_eq!(RuntimeError::Ok.code(), 0);
        assert_eq!(RuntimeError::OutOfMemory.code(), 1);
        assert_eq!(RuntimeError::Closed.code(), 9);
        assert_eq!(RuntimeError::TimedOut.code(), 10);
    }

    #[test]
    fn transient_classification() {
        assert!(RuntimeError::WouldBlock.is_transient());
        assert!(RuntimeError::TimedOut.is_transient());
        assert!(!RuntimeError::OutOfMemory.is_transient());
    }

    #[test]
    fn fatal_classification() {
        assert!(RuntimeError::OutOfMemory.is_fatal());
        assert!(RuntimeError::StackOverflow.is_fatal());
        assert!(!RuntimeError::WouldBlock.is_fatal());
    }

    #[test]
    fn display_round_trip() {
        for e in [
            RuntimeError::Ok,
            RuntimeError::OutOfMemory,
            RuntimeError::Disconnected,
            RuntimeError::WouldBlock,
            RuntimeError::Closed,
            RuntimeError::TimedOut,
        ] {
            assert!(render_len(e) > 0);
        }
    }

    #[test]
    fn from_unit_is_ok() {
        let e: RuntimeError = ().into();
        assert_eq!(e, RuntimeError::Ok);
    }

    #[test]
    fn default_is_ok() {
        assert_eq!(RuntimeError::default(), RuntimeError::Ok);
    }
}
