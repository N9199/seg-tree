mod iterative;
mod lazy;
mod persistent;
mod persistent_lazy;

pub use self::{iterative::SegmentTree, lazy::LazySegmentTree, persistent::PersistentSegmentTree, persistent_lazy::LazyPersistentSegmentTree};
