mod iterative;
mod lazy;
mod persistent;
pub mod persistent_lazy;

pub use self::{iterative::IterativeSegmentTree, lazy::LazySegmentTree, persistent::PersistentSegmentTree};
