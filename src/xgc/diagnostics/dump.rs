//! Heap dump: produce a human-readable summary of heap state.
//!
//! The dump is written to a fixed-size buffer as a sequence of lines.
//! Intended for `panic!`-style diagnostics, not production logging.

use crate::xgc::region::table::RegionTable;

const DUMP_LINE_LEN: usize = 64;
const DUMP_MAX_LINES: usize = 64;

/// Buffered heap dump.
pub struct HeapDump {
    lines: [[u8; DUMP_LINE_LEN]; DUMP_MAX_LINES],
    line_count: usize,
}

impl HeapDump {
    /// Construct an empty dump.
    pub const fn new() -> Self {
        Self {
            lines: [[0u8; DUMP_LINE_LEN]; DUMP_MAX_LINES],
            line_count: 0,
        }
    }

    /// Append a line to the dump. Truncates if full.
    pub fn write_line(&mut self, line: &str) {
        if self.line_count >= DUMP_MAX_LINES {
            return;
        }
        let bytes = line.as_bytes();
        let n = bytes.len().min(DUMP_LINE_LEN);
        self.lines[self.line_count][..n].copy_from_slice(&bytes[..n]);
        self.line_count += 1;
    }

    /// Number of lines written.
    pub fn line_count(&self) -> usize {
        self.line_count
    }

    /// Get a line as bytes.
    pub fn get_line(&self, idx: usize) -> &[u8] {
        if idx >= self.line_count {
            return &[];
        }
        let len = self.lines[idx]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(DUMP_LINE_LEN);
        &self.lines[idx][..len]
    }
}

impl Default for HeapDump {
    fn default() -> Self {
        Self::new()
    }
}

/// Populate a `HeapDump` with a summary of the given region table.
pub fn dump_region_table(table: &RegionTable, dump: &mut HeapDump) {
    dump.write_line("=== Region Table Dump ===");
    for i in 0..table.len().min(DUMP_MAX_LINES - 1) {
        if let Some(r) = table.get(i) {
            let line = if r.is_available() {
                "R: free"
            } else {
                "R: bound"
            };
            dump.write_line(line);
        }
    }
}
