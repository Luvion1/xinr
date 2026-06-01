//! NUMA topology: maps regions to nodes and tracks per-node capacity.

use crate::xgc::numa::node::NodeId;

const MAX_NODES: usize = 16;

/// One NUMA node's capacity.
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeCapacity {
    pub node: NodeId,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub region_count: u32,
}

/// NUMA topology.
pub struct NumaTopology {
    nodes: [NodeCapacity; MAX_NODES],
    active: u8,
}

impl NumaTopology {
    /// Construct an empty topology.
    pub const fn new() -> Self {
        Self {
            nodes: [NodeCapacity {
                node: NodeId::LOCAL,
                total_bytes: 0,
                used_bytes: 0,
                region_count: 0,
            }; MAX_NODES],
            active: 0,
        }
    }

    /// Register a node.
    pub fn register(&mut self, id: NodeId, total_bytes: u64) {
        if (self.active as usize) >= MAX_NODES {
            return;
        }
        self.nodes[self.active as usize] = NodeCapacity {
            node: id,
            total_bytes,
            used_bytes: 0,
            region_count: 0,
        };
        self.active += 1;
    }

    /// Get capacity for a node.
    pub fn capacity(&self, id: NodeId) -> Option<&NodeCapacity> {
        self.nodes[..self.active as usize]
            .iter()
            .find(|n| n.node == id)
    }

    /// Get the node with the most free space.
    pub fn most_free(&self) -> Option<NodeId> {
        self.nodes[..self.active as usize]
            .iter()
            .max_by_key(|n| n.total_bytes.saturating_sub(n.used_bytes))
            .map(|n| n.node)
    }

    /// Number of registered nodes.
    pub fn node_count(&self) -> u8 {
        self.active
    }

    /// Reserve `bytes` on `node`.
    pub fn reserve(&mut self, node: NodeId, bytes: u64) -> bool {
        for n in self.nodes[..self.active as usize].iter_mut() {
            if n.node == node {
                if n.total_bytes.saturating_sub(n.used_bytes) >= bytes {
                    n.used_bytes += bytes;
                    n.region_count += 1;
                    return true;
                }
                return false;
            }
        }
        false
    }

    /// Release `bytes` on `node`.
    pub fn release(&mut self, node: NodeId, bytes: u64) {
        for n in self.nodes[..self.active as usize].iter_mut() {
            if n.node == node {
                n.used_bytes = n.used_bytes.saturating_sub(bytes);
                if n.region_count > 0 {
                    n.region_count -= 1;
                }
                return;
            }
        }
    }
}

impl Default for NumaTopology {
    fn default() -> Self {
        Self::new()
    }
}
