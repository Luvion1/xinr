//! Benchmark skeleton: defines macros for micro-benchmarks.
//!
//! Compiled only under `#[cfg(bench)]` (nightly). On stable, the macro
//! is a no-op.
//!
//! # Example
//!
//! ```ignore
//! use xinr::bench::bench;
//! use xinr::sync::channel::BoundedChannel;
//!
//! bench!("channel_throughput", || {
//!     let mut ch: BoundedChannel<u32, 64> = BoundedChannel::new();
//!     for i in 0..1000 {
//!         ch.try_send(i).unwrap();
//!         let _ = ch.try_recv().unwrap();
//!     }
//! });
//! ```

/// Run a micro-benchmark. Stable builds expand to an inline measurement
/// loop. Nightly with `#[bench]` support can switch to a real harness.
#[macro_export]
macro_rules! bench {
    ($name:literal, $body:expr) => {{
        let start = $crate::bench::_timestamp();
        let result = { $body };
        let elapsed = $crate::bench::_timestamp().saturating_sub(start);
        $crate::bench::_record($name, elapsed);
        result
    }};
}

/// Run a series of benchmark variants sharing a prefix, recording each
/// with a `<prefix>::<variant>` label and returning the total elapsed
/// ticks across all variants.
///
/// ```ignore
/// bench_group!("throughput", {
///     "send_only"   => || ch.try_send(i).unwrap(),
///     "recv_only"   => || ch.try_recv().unwrap(),
///     "round_trip"  => || { ch.try_send(i).unwrap(); ch.try_recv().unwrap(); },
/// });
/// ```
#[macro_export]
macro_rules! bench_group {
    ($prefix:literal, { $( $variant:literal => $body:expr ),+ $(,)? }) => {{
        let total_start = $crate::bench::_timestamp();
        $({
            let __start = $crate::bench::_timestamp();
            let __r = { $body };
            let __elapsed = $crate::bench::_timestamp().saturating_sub(__start);
            $crate::bench::_record(concat!($prefix, "::", $variant), __elapsed);
            let _ = __r;
        })+
        $crate::bench::_timestamp().saturating_sub(total_start)
    }};
}

/// Mark a function as a benchmark target. Stable builds expand to
/// `#[test] #[ignore]` so `cargo test` doesn't run them by default.
#[macro_export]
macro_rules! bench_test {
    ($name:ident, $body:expr) => {
        #[test]
        #[ignore = "benchmark; run with --ignored"]
        fn $name() {
            let _ = $body;
        }
    };
}

/// Monotonic timestamp in nanoseconds. On `no_std` without atomics we
/// return a coarse counter incremented on every call (useful only for
/// relative comparisons).
#[inline]
pub fn _timestamp() -> u64 {
    static mut COUNTER: u64 = 0;
    unsafe {
        COUNTER = COUNTER.wrapping_add(1);
        COUNTER
    }
}

/// Record a benchmark result. On `no_std` this is a no-op; a host
/// binary can override via the `bench-host` feature.
#[inline]
pub fn _record(_name: &str, _elapsed: u64) {}

/// Time a single expression and return `(elapsed_ticks, result)`.
/// Useful when you need both the result and the timing.
///
/// ```ignore
/// let (ticks, value) = time_it!(ch.try_send(42).unwrap());
/// ```
#[macro_export]
macro_rules! time_it {
    ($body:expr) => {{
        let __start = $crate::bench::_timestamp();
        let __r = { $body };
        let __elapsed = $crate::bench::_timestamp().saturating_sub(__start);
        (__elapsed, __r)
    }};
}
