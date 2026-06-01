//! NUMA placement: node identifiers and topology.

pub mod node;
pub mod topology;

pub use node::NodeId;
pub use topology::{NodeCapacity, NumaTopology};
