//! Trace trait: walk an object's outgoing references.

use crate::xgc::colored::ColoredPtr;

/// Visitor that accumulates references.
pub trait Visitor {
    /// Visit one reference.
    fn visit(&mut self, r: ColoredPtr);
}

/// Implemented by heap-allocated types to participate in GC tracing.
pub trait Trace {
    /// Walk all outgoing colored pointers.
    fn trace(&self, _v: &mut dyn Visitor);
}

impl Trace for u8 {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for u16 {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for u32 {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for u64 {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for usize {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for i8 {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for i16 {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for i32 {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for i64 {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for isize {
    fn trace(&self, _: &mut dyn Visitor) {}
}
impl Trace for bool {
    fn trace(&self, _: &mut dyn Visitor) {}
}
