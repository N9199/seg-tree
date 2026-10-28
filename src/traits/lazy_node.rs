use core::range::RangeInclusive;

use super::Node;

/// Required trait by nodes of lazy segment trees.
///
/// A pending update represents work not yet reflected in the node's aggregate.
/// Implementations commonly store it in an `Option`. Applying an update to a
/// segment must agree with applying it to the children and combining their results.
/// See [Implementors](LazyNode#implementors) for some example implementations.
pub trait LazyNode: Node {
    /// An update that can be composed with other updates and applied to a segment.
    type Update: Clone;

    /// Borrows the pending update without consuming it or changing the aggregate.
    fn pending(&self) -> Option<&Self::Update>;

    /// Merge an incoming update into this node's pending slot — composing
    /// with anything already pending, adopting it otherwise. Existing pending work
    /// happens before the incoming update; composition need not be commutative.
    /// Does NOT touch `value`; this node hasn't been visited yet.
    fn defer(&mut self, update: &Self::Update);

    /// Applies and consumes the pending update for the supplied inclusive range.
    ///
    /// On successful return, the aggregate includes the update and [`pending`](Self::pending)
    /// returns `None`. Does nothing if no update is pending.
    fn apply_pending(&mut self, range: RangeInclusive<usize>);
}
