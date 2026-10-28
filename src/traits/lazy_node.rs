use core::range::RangeInclusive;

use super::Node;

/// Required trait by nodes of lazy segment trees.
///
/// It's defined as an interface for the operations needed on the `lazy_value`.
/// It is recommended to implement it using an Option type.
/// See [Implementors](LazyNode#implementors) for some example implementations.
pub trait LazyNode: Node {
    type Update: Clone;

    /// Peeks at this node's pending update
    fn pending(&self) -> Option<&Self::Update>;

    /// Merge an incoming update into this node's pending slot — composing
    /// with anything already pending, adopting it otherwise. Does NOT
    /// touch `value`; this node hasn't been visited yet.
    fn defer(&mut self, update: &Self::Update);

    /// Consumes own pending update if any.
    fn apply_pending(&mut self, range: RangeInclusive<usize>);
}
