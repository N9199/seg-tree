//! Storage-lifetime regressions using only the public tree API.
//!
//! Construction panics and a single node destructor panic must clean up every
//! initialized node.

use std::{
    cell::Cell,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

use seg_tree::{
    Iterative, LazyPersistent, LazyRecursive, Persistent, Recursive,
    traits::{LazyNode, Node},
};

#[derive(Default)]
struct Counts {
    created: Cell<usize>,
    dropped: Cell<usize>,
    clone_budget: Cell<Option<usize>>,
    combine_budget: Cell<Option<usize>>,
    drop_budget: Cell<Option<usize>>,
    panic_on_drop: Cell<bool>,
    panic_on_apply: Cell<bool>,
}

#[derive(Clone)]
struct Value {
    number: usize,
    counts: Rc<Counts>,
}

struct TrackedNode {
    value: Value,
    pending: Option<usize>,
    _allocation: Box<usize>,
}

fn consume_budget(budget: &Cell<Option<usize>>) {
    if let Some(remaining) = budget.get() {
        budget.set(remaining.checked_sub(1));
        assert!(remaining != 0, "injected node callback panic");
    }
}

impl Clone for TrackedNode {
    fn clone(&self) -> Self {
        consume_budget(&self.value.counts.clone_budget);
        Self::initialize(&self.value)
    }
}

impl Node for TrackedNode {
    type Value = Value;

    fn initialize(value: &Value) -> Self {
        value.counts.created.set(value.counts.created.get() + 1);
        Self {
            value: value.clone(),
            pending: None,
            _allocation: Box::new(value.number),
        }
    }

    fn combine(a: &Self, b: &Self) -> Self {
        consume_budget(&a.value.counts.combine_budget);
        Self::initialize(&Value {
            number: a.value.number + b.value.number,
            counts: Rc::clone(&a.value.counts),
        })
    }

    fn value(&self) -> &Value {
        &self.value
    }
}

impl LazyNode for TrackedNode {
    type Update = usize;

    fn pending(&self) -> Option<&usize> {
        self.pending.as_ref()
    }

    fn defer(&mut self, update: &usize) {
        *self.pending.get_or_insert(0) += update;
    }

    fn apply_pending(&mut self, range: core::range::RangeInclusive<usize>) {
        if let Some(update) = self.pending.take() {
            assert!(
                !self.value.counts.panic_on_apply.replace(false),
                "injected update panic"
            );
            self.value.number += update * (range.last - range.start + 1);
        }
    }
}

impl Drop for TrackedNode {
    fn drop(&mut self) {
        let counts = &self.value.counts;
        counts.dropped.set(counts.dropped.get() + 1);
        consume_budget(&counts.drop_budget);
        assert!(!counts.panic_on_drop.replace(false), "injected drop panic");
    }
}

fn inputs(counts: &Rc<Counts>, n: usize) -> Vec<TrackedNode> {
    (1..=n)
        .map(|number| {
            TrackedNode::initialize(&Value {
                number,
                counts: Rc::clone(counts),
            })
        })
        .collect()
}

fn assert_all_dropped(counts: &Counts) {
    assert_eq!(
        counts.dropped.get(),
        counts.created.get(),
        "every constructed node must be destroyed after construction or cleanup"
    );
}

thread_local! {
    static ZST_CREATED: Cell<usize> = const { Cell::new(0) };
    static ZST_DROPPED: Cell<usize> = const { Cell::new(0) };
    static ZST_DROP_BUDGET: Cell<Option<usize>> = const { Cell::new(None) };
}

struct ZeroSizedNode;

impl Clone for ZeroSizedNode {
    fn clone(&self) -> Self {
        Self::initialize(&())
    }
}

impl Node for ZeroSizedNode {
    type Value = ();

    fn initialize(_: &()) -> Self {
        ZST_CREATED.with(|count| count.set(count.get() + 1));
        Self
    }

    fn combine(_: &Self, _: &Self) -> Self {
        Self::initialize(&())
    }

    fn value(&self) -> &() {
        &()
    }
}

impl Drop for ZeroSizedNode {
    fn drop(&mut self) {
        ZST_DROPPED.with(|count| count.set(count.get() + 1));
        ZST_DROP_BUDGET.with(|budget| {
            if let Some(remaining) = budget.get() {
                budget.set(remaining.checked_sub(1));
                assert!(remaining != 0, "injected zero-sized node destructor panic");
            }
        });
    }
}

impl LazyNode for ZeroSizedNode {
    type Update = ();

    fn pending(&self) -> Option<&()> {
        None
    }

    fn defer(&mut self, _: &()) {}

    fn apply_pending(&mut self, _: core::range::RangeInclusive<usize>) {}
}

macro_rules! lifetime_tests {
    ($module:ident, $tree:ident) => {
        mod $module {
            use super::*;

            #[test]
            fn zero_sized_nodes_build_and_drop() {
                assert_eq!(std::mem::size_of::<ZeroSizedNode>(), 0);
                for n in [0, 1, 3] {
                    drop($tree::build(&vec![ZeroSizedNode; n]));
                }
            }

            #[test]
            fn zero_sized_destructor_panics_clean_up_all_initialized_nodes() {
                assert_eq!(std::mem::size_of::<ZeroSizedNode>(), 0);
                for n in [1, 3] {
                    for panic_at in 0..2 * n - 1 {
                        ZST_CREATED.with(|count| count.set(0));
                        ZST_DROPPED.with(|count| count.set(0));
                        ZST_DROP_BUDGET.with(|budget| budget.set(None));
                        let values: Vec<_> =
                            (0..n).map(|_| ZeroSizedNode::initialize(&())).collect();
                        let tree = $tree::build(&values);
                        ZST_DROP_BUDGET.with(|budget| budget.set(Some(panic_at)));
                        let result = catch_unwind(AssertUnwindSafe(|| drop(tree)));
                        assert!(result.is_err());
                        drop(values);
                        let created = ZST_CREATED.with(Cell::get);
                        let dropped = ZST_DROPPED.with(Cell::get);
                        assert_eq!(created, 3 * n - 1);
                        assert_eq!(dropped, created, "size {n}, panic at {panic_at}");
                    }
                }
            }

            #[test]
            fn all_initialized_nodes_are_dropped_exactly_once() {
                for n in [0, 1, 2, 3, 5, 8, 17] {
                    let counts = Rc::new(Counts::default());
                    let values = inputs(&counts, n);
                    let tree = $tree::build(&values);
                    if n != 0 {
                        let result = tree.query(0, n - 1).unwrap();
                        assert_eq!(result.value().number, n * (n + 1) / 2);
                        drop(result);
                    }
                    drop(tree);
                    drop(values);
                    assert_all_dropped(&counts);
                }
            }

            #[test]
            fn clone_panics_clean_up_all_constructed_nodes() {
                for n in [1, 2, 3, 5, 8, 17] {
                    for fail_at in 0..n {
                        let counts = Rc::new(Counts::default());
                        let values = inputs(&counts, n);
                        counts.clone_budget.set(Some(fail_at));
                        let result = catch_unwind(AssertUnwindSafe(|| $tree::build(&values)));
                        assert!(result.is_err(), "clone panic at call {fail_at}, size {n}");
                        drop(values);
                        assert_all_dropped(&counts);
                    }
                }
            }

            #[test]
            fn combine_panics_clean_up_all_constructed_nodes() {
                for n in [1, 2, 3, 5, 8, 17] {
                    for fail_at in 0..n - 1 {
                        let counts = Rc::new(Counts::default());
                        let values = inputs(&counts, n);
                        counts.combine_budget.set(Some(fail_at));
                        let result = catch_unwind(AssertUnwindSafe(|| $tree::build(&values)));
                        assert!(result.is_err(), "combine panic at call {fail_at}, size {n}");
                        drop(values);
                        assert_all_dropped(&counts);
                    }
                }
            }

            #[test]
            fn destructor_panics_clean_up_all_initialized_nodes() {
                for n in [1, 2, 3, 5, 8, 17] {
                    for panic_at in 0..2 * n - 1 {
                        let counts = Rc::new(Counts::default());
                        let values = inputs(&counts, n);
                        let tree = $tree::build(&values);
                        counts.drop_budget.set(Some(panic_at));
                        let result = catch_unwind(AssertUnwindSafe(|| drop(tree)));
                        assert!(
                            result.is_err(),
                            "expected destructor panic at node {panic_at}, size {n}"
                        );
                        drop(values);
                        assert_all_dropped(&counts);
                    }
                }
            }
        }
    };
}

lifetime_tests!(iterative, Iterative);
lifetime_tests!(recursive, Recursive);
lifetime_tests!(lazy_recursive, LazyRecursive);

macro_rules! persistent_lifetime_tests {
    ($module:ident, $tree:ident) => {
        mod $module {
            use super::*;

            #[test]
            fn zero_sized_nodes_build_and_drop() {
                for n in [0, 1, 3] {
                    drop($tree::build(&vec![ZeroSizedNode; n]));
                }
            }

            #[test]
            fn dense_arena_drops_every_initialized_node() {
                for n in [0, 1, 3, 8, 17] {
                    let counts = Rc::new(Counts::default());
                    let values = inputs(&counts, n);
                    // LazyPersistent queries mutate the arena; Persistent queries do not.
                    #[allow(unused_mut)]
                    let mut tree = $tree::build(&values);
                    if n != 0 {
                        let result = tree.query(0, 0, n - 1).unwrap();
                        assert_eq!(result.value().number, n * (n + 1) / 2);
                        drop(result);
                    }
                    drop(tree);
                    drop(values);
                    assert_all_dropped(&counts);
                }
            }

            #[test]
            fn dense_arena_cleans_up_after_constructor_panics() {
                for fail_in_clone in [true, false] {
                    let counts = Rc::new(Counts::default());
                    let values = inputs(&counts, 3);
                    if fail_in_clone {
                        counts.clone_budget.set(Some(1));
                    } else {
                        counts.combine_budget.set(Some(0));
                    }
                    let result = catch_unwind(AssertUnwindSafe(|| $tree::build(&values)));
                    assert!(result.is_err());
                    drop(values);
                    assert_all_dropped(&counts);
                }
            }
        }
    };
}

persistent_lifetime_tests!(persistent, Persistent);
persistent_lifetime_tests!(lazy_persistent, LazyPersistent);

#[test]
fn lazy_tree_drops_safely_after_a_public_update_panics() {
    let counts = Rc::new(Counts::default());
    let values = inputs(&counts, 3);
    let mut tree = LazyRecursive::build(&values);
    counts.panic_on_apply.set(true);
    let result = catch_unwind(AssertUnwindSafe(|| tree.update(0, 2, &1)));
    assert!(result.is_err());
    drop(tree);
    drop(values);
    assert_all_dropped(&counts);
}

#[test]
fn lazy_tree_drops_safely_after_a_replaced_node_destructor_panics() {
    let counts = Rc::new(Counts::default());
    let values = inputs(&counts, 3);
    let mut tree = LazyRecursive::build(&values);
    counts.panic_on_drop.set(true);
    let result = catch_unwind(AssertUnwindSafe(|| tree.update(0, 0, &1)));
    assert!(result.is_err());
    drop(tree);
    drop(values);
    assert_all_dropped(&counts);
}

#[test]
fn lazy_tree_drops_safely_after_poisoned_mutex_and_destructor_panic() {
    let counts = Rc::new(Counts::default());
    let values = inputs(&counts, 3);
    let mut tree = LazyRecursive::build(&values);
    counts.panic_on_apply.set(true);
    let update = catch_unwind(AssertUnwindSafe(|| tree.update(0, 2, &1)));
    assert!(update.is_err());
    counts.drop_budget.set(Some(0));
    let result = catch_unwind(AssertUnwindSafe(|| drop(tree)));
    assert!(result.is_err());
    drop(values);
    assert_all_dropped(&counts);
}

// Keep overflow regressions Miri-only so a broken bounds check never causes
// native undefined behavior. Disable debug assertions and overflow checks.
#[cfg(all(miri, not(debug_assertions)))]
#[test]
fn iterative_wrapping_update_index_preserves_initialized_storage() {
    let counts = Rc::new(Counts::default());
    let values = inputs(&counts, 1);
    let mut tree = Iterative::build(&values);
    let replacement = Value {
        number: 2,
        counts: Rc::clone(&counts),
    };
    // Checked rejection and a no-op must both preserve the initialized leaf.
    let _ = catch_unwind(AssertUnwindSafe(|| tree.update(usize::MAX, &replacement)));
    assert_eq!(tree.query(0, 0).unwrap().value().number, 1);
    drop(tree);
    drop(values);
    assert_all_dropped(&counts);
}

#[cfg(all(miri, not(debug_assertions)))]
#[test]
fn iterative_wrapping_query_indices_are_normalized_without_reading_uninitialized_storage() {
    let counts = Rc::new(Counts::default());
    let values = inputs(&counts, 1);
    let tree = Iterative::build(&values);
    assert!(tree.query(usize::MAX, 0).is_none());
    assert!(tree.query(usize::MAX, usize::MAX).is_none());
    assert_eq!(tree.query(0, usize::MAX).unwrap().value().number, 1);
    drop(tree);
    drop(values);
    assert_all_dropped(&counts);
}
