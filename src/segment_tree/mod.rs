mod iterative;
mod lazy;
mod persistent;
mod lazy_persistent;
mod recursive;

pub use self::{
    iterative::SegmentTree, lazy::Lazy, persistent::Persistent,
    lazy_persistent::LazyPersistent, recursive::Recursive,
};
