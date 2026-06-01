//! NUMA node id: opaque identifier for a non-uniform memory access node.

/// Opaque NUMA node identifier. Node 0 is the local node by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NodeId(pub u8);

impl NodeId {
    /// Node 0 (the local node on most single-socket systems).
    pub const LOCAL: NodeId = NodeId(0);
    /// Sentinel for "any node" (no preference).
    pub const ANY: NodeId = NodeId(u8::MAX);

    /// Construct a node id.
    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    /// Raw node number.
    pub fn raw(self) -> u8 {
        self.0
    }

    /// Whether the node id is valid (not ANY).
    pub fn is_valid(self) -> bool {
        self.0 != u8::MAX
    }
}
