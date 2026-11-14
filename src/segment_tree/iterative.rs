use core::mem::{DropGuard, MaybeUninit};

use crate::{internal_utils::dbg_utils::as_dbg_tree, traits::Node};

/// Segment tree with range queries and point updates.
///
/// It uses `O(n)` space, assuming that each node uses `O(1)` space.
/// Note if you need to use `lower_bound`, just use [`Recursive`](crate::segment_tree::Recursive) it uses double the memory though and it's less performant.
pub struct Iterative<T> {
    // Slot zero is uninitialized; every other slot is initialized in a built tree.
    nodes: Vec<MaybeUninit<T>>,
    n: usize,
}

impl<T> Iterative<T>
where
    T: Node + Clone,
{
    /// Builds segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// It has time complexity of `O(n*log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut nodes: Vec<MaybeUninit<T>> = Vec::with_capacity(2 * n);
        // SAFETY: Uninitialized slots are valid MaybeUninit<T> values.
        unsafe { nodes.set_len(2 * n) };
        let mut guard = DropGuard::new((nodes, n..n), |(mut nodes, initialized)| {
            for node in &mut nodes[initialized] {
                // SAFETY: The range advances only after a slot is initialized.
                unsafe { node.assume_init_drop() };
            }
        });
        for i in 0..n {
            guard.0[i + n].write(values[i].clone());
            guard.1.end = i + n + 1;
        }
        for i in (1..n).rev() {
            let (bottom_nodes, top_nodes) = guard.0.split_at_mut(i + 1);
            bottom_nodes[i].write(Node::combine(
                unsafe { top_nodes[i - 1].assume_init_ref() },
                unsafe { top_nodes[i].assume_init_ref() },
            ));
            guard.1.start = i;
        }

        let (nodes, _) = DropGuard::dismiss(guard);
        Self { nodes, n }
    }

    /// Sets the i-th element of the segment tree to value T and update the segment tree correspondingly.
    /// Does nothing if `i` is outside the original slice, including an empty tree.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    pub fn update(&mut self, i: usize, value: &<T as Node>::Value) {
        if i >= self.n {
            return;
        }
        let mut i = i;
        i += self.n;
        let value = Node::initialize(value);
        // SAFETY: The bounds check selects an initialized leaf.
        unsafe { *self.nodes[i].assume_init_mut() = value };
        i >>= 1;
        while i > 0 {
            // SAFETY: Both children of this internal node are initialized.
            let value = unsafe {
                Node::combine(
                    self.nodes[2 * i].assume_init_ref(),
                    self.nodes[2 * i + 1].assume_init_ref(),
                )
            };
            // SAFETY: This internal slot is initialized. Assignment keeps it
            // initialized even if the old node's destructor panics.
            unsafe { *self.nodes[i].assume_init_mut() = value };
            i >>= 1;
        }
    }

    /// Returns the aggregate of the inclusive range `[left, right]` intersected
    /// with the original slice. Returns `None` if the intersection or tree is empty.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    #[must_use]
    pub fn query(&self, l: usize, r: usize) -> Option<T> {
        if self.n == 0 || l > r || l >= self.n {
            return None;
        }
        let (mut l, mut r) = (l, r.min(self.n - 1));
        let mut ans_left = None;
        let mut ans_right = None;
        l += self.n;
        r += self.n + 1;
        while l < r {
            if l & 1 != 0 {
                // SAFETY: The query traversal selects an initialized slot.
                let selected = unsafe { self.nodes[l].assume_init_ref() };
                ans_left = Some(
                    ans_left
                        .map_or_else(|| selected.clone(), |node| Node::combine(&node, selected)),
                );
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                // SAFETY: The query traversal selects an initialized slot.
                let selected = unsafe { self.nodes[r].assume_init_ref() };
                ans_right = Some(
                    ans_right
                        .map_or_else(|| selected.clone(), |node| Node::combine(selected, &node)),
                );
            }
            l >>= 1;
            r >>= 1;
        }
        match (ans_left, ans_right) {
            (Some(ans_left), Some(ans_right)) => Some(Node::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }
}

impl<T> Drop for Iterative<T> {
    fn drop(&mut self) {
        if let Some(nodes) = self.nodes.get_mut(1..) {
            // SAFETY: Every slot except zero is initialized, even after an update
            // panics, and MaybeUninit<T> has the same layout as T. Slice drop glue
            // also destroys the remaining elements if one destructor panics.
            unsafe { core::ptr::drop_in_place(core::ptr::from_mut(nodes) as *mut [T]) };
        }
    }
}

impl<T> Iterative<T>
where
    T: Node + core::fmt::Debug,
{
    fn dbg_visitor<'a>(
        n: usize,
        f: &mut dyn FnMut(usize, usize, &'a T),
        nodes: &'a [MaybeUninit<T>],
    ) {
        let mut segments = vec![(0, 0); 2 * n];
        for i in 0..n {
            segments[i + n] = (i, i);
            // SAFETY: Each leaf is initialized.
            f(i, i, unsafe { nodes[n + i].assume_init_ref() });
        }
        let helper = |(a1, b1): (usize, usize), (a2, b2)| (a1.min(a2), b1.max(b2));
        for i in (1..n).rev() {
            segments[i] = helper(segments[2 * i], segments[2 * i + 1]);
            // SAFETY: Each internal slot in 1..n is initialized.
            f(segments[i].0, segments[i].1, unsafe {
                nodes[i].assume_init_ref()
            });
        }
    }
}

impl<T> core::fmt::Debug for Iterative<T>
where
    T: Node + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Iterative")
            .field("n", &self.n)
            .field(
                "nodes",
                &as_dbg_tree(&self.nodes, |nodes, f| Self::dbg_visitor(self.n, f, nodes)),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{internal_utils::test_utils::assert_range_matches, traits::Node, utils::Min};

    use super::Iterative;

    #[test]
    fn query_preserves_max_subarray_aggregate() {
        use crate::utils::MaxSubArraySum;

        let nodes: Vec<_> = [0, 0, 4, -5, -5, 4, 0, 0]
            .iter()
            .map(MaxSubArraySum::initialize)
            .collect();
        let tree = Iterative::build(&nodes);
        assert_eq!(*tree.query(2, 5).unwrap().value(), 4);
    }

    #[test]
    fn query_preserves_hidden_node_state() {
        use crate::utils::MaxSubArraySum;

        let nodes: Vec<_> = [4, -5].iter().map(MaxSubArraySum::initialize).collect();
        let expected = MaxSubArraySum::combine(&nodes[0], &nodes[1]);
        let tree = Iterative::build(&nodes);
        assert_eq!(tree.query(0, 1).unwrap(), expected);
    }

    #[test]
    fn all_ranges_preserve_aggregate_state_after_updates() {
        use crate::utils::MaxSubArraySum;

        for mut values in [
            vec![4],
            vec![4, -5],
            vec![-4, -3, -2],
            vec![4, -5, -5, 4, 2],
            vec![0, 0, 4, -5, -5, 4, 0, 0],
            vec![3, -4, 2, 0, -6, 5, -1, 4, -2],
        ] {
            let nodes: Vec<_> = values.iter().map(MaxSubArraySum::initialize).collect();
            let mut tree = Iterative::build(&nodes);
            for update in [None, Some((0, -7)), Some((values.len() - 1, 6))] {
                if let Some((index, value)) = update {
                    values[index] = value;
                    tree.update(index, &value);
                }
                assert_range_matches(
                    &values,
                    |left, right| tree.query(left, right).unwrap(),
                    |range| {
                        let mut leaves = range.iter().map(MaxSubArraySum::initialize);
                        let first = leaves.next().unwrap();
                        leaves.fold(first, |a, b| MaxSubArraySum::combine(&a, &b))
                    },
                );
            }
        }
    }

    #[test]
    fn query_preserves_noncommutative_order() {
        use crate::utils::Sum;

        #[derive(Clone, Copy, Debug, PartialEq)]
        struct Digits {
            value: usize,
            len: u32,
        }

        impl std::ops::Add for Digits {
            type Output = Self;

            fn add(self, rhs: Self) -> Self {
                Self {
                    value: self.value * 10_usize.pow(rhs.len) + rhs.value,
                    len: self.len + rhs.len,
                }
            }
        }

        let mut values: Vec<_> = (1..=7).map(|value| Digits { value, len: 1 }).collect();
        let nodes: Vec<_> = values.iter().map(Sum::initialize).collect();
        let mut tree = Iterative::build(&nodes);
        for update in [None, Some((0, 9)), Some((3, 0)), Some((6, 8))] {
            if let Some((index, value)) = update {
                values[index] = Digits { value, len: 1 };
                tree.update(index, &values[index]);
            }
            assert_range_matches(
                &values,
                |left, right| *tree.query(left, right).unwrap().value(),
                |range| {
                    range
                        .iter()
                        .copied()
                        .fold(Digits { value: 0, len: 0 }, |a, b| Digits {
                            value: a.value * 10 + b.value,
                            len: a.len + 1,
                        })
                },
            );
        }
    }

    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Iterative::build(&nodes);
        assert!(segment_tree.query(0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Iterative::build(&nodes);
        assert!(segment_tree.query(10, 0).is_none());
    }
    #[test]
    fn update_out_of_bounds_does_nothing() {
        let mut tree = Iterative::build(&[Min::initialize(&0usize)]);
        tree.update(1, &1);
        assert_eq!(*tree.query(0, 0).unwrap().value(), 0);
    }

    #[test]
    fn query_out_of_bounds_left_returns_none() {
        let tree = Iterative::build(&[Min::initialize(&0usize)]);
        assert!(tree.query(1, 0).is_none());
        assert!(tree.query(1, usize::MAX).is_none());
    }

    #[test]
    fn query_out_of_bounds_right_is_normalized() {
        let tree = Iterative::build(&[Min::initialize(&0usize)]);
        assert_eq!(*tree.query(0, usize::MAX).unwrap().value(), 0);
    }

    #[test]
    fn update_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = Iterative::build(&nodes);
        let value = 20;
        segment_tree.update(0, &value);
        assert_eq!(segment_tree.query(0, 0).unwrap().value(), &value);
    }
    #[test]
    fn query_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Iterative::build(&nodes);
        for i in 0..10 {
            assert_eq!(segment_tree.query(i, 10).unwrap().value(), &i);
        }
    }

    #[test]
    fn drop_empty_tree_does_not_panic() {
        drop(Iterative::<Min<usize>>::build(&[]));
    }

    #[cfg(miri)]
    #[test]
    fn miri_drop_owned_nodes() {
        #[derive(Clone)]
        struct OwnedNode(Box<usize>);

        impl Node for OwnedNode {
            type Value = usize;

            fn initialize(value: &usize) -> Self {
                Self(Box::new(*value))
            }

            fn combine(a: &Self, b: &Self) -> Self {
                Self::initialize(&(*a.0).min(*b.0))
            }

            fn value(&self) -> &usize {
                &self.0
            }
        }

        let values = [OwnedNode::initialize(&42)];
        drop(Iterative::build(&values));
    }

    #[test]
    fn update_drop_panic_leaves_tree_safe_to_destroy() {
        use core::sync::atomic::{AtomicUsize, Ordering};
        use std::panic::{AssertUnwindSafe, catch_unwind};

        static PANIC_VALUE: AtomicUsize = AtomicUsize::new(0);

        #[derive(Clone)]
        struct OwnedNode(Box<usize>);

        impl Node for OwnedNode {
            type Value = usize;

            fn initialize(value: &usize) -> Self {
                Self(Box::new(*value))
            }

            fn combine(a: &Self, b: &Self) -> Self {
                Self::initialize(&(*a.0 + *b.0))
            }

            fn value(&self) -> &usize {
                &self.0
            }
        }

        impl Drop for OwnedNode {
            fn drop(&mut self) {
                if PANIC_VALUE
                    .compare_exchange(*self.0, 0, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    panic!("old node destructor panicked");
                }
            }
        }

        // Exercise replacement of both a leaf and an internal node. Box makes
        // stale-slot destruction detectable as a double free under Miri.
        for (n, panic_value) in [(1, 1), (2, 3)] {
            let values: Vec<_> = (1..=n).map(|value| OwnedNode::initialize(&value)).collect();
            let mut tree = Iterative::build(&values);
            PANIC_VALUE.store(panic_value, Ordering::SeqCst);
            assert!(catch_unwind(AssertUnwindSafe(|| tree.update(0, &4))).is_err());
            assert_eq!(tree.query(0, 0).unwrap().value(), &4);
            drop(tree);
        }
    }

    #[test]
    fn dbg_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = Iterative::build(&nodes);
        segment_tree.update(0, &2);
        let dbg = format!("{segment_tree:?}");
        let expected = "Iterative { n: 11, nodes: {[0, 0]: Min { value: 2 }, [1, 1]: Min { value: 1 }, [2, 2]: Min { value: 2 }, [3, 3]: Min { value: 3 }, [4, 4]: Min { value: 4 }, [5, 5]: Min { value: 5 }, [6, 6]: Min { value: 6 }, [7, 7]: Min { value: 7 }, [8, 8]: Min { value: 8 }, [9, 9]: Min { value: 9 }, [10, 10]: Min { value: 10 }, [9, 10]: Min { value: 9 }, [7, 8]: Min { value: 7 }, [5, 6]: Min { value: 5 }, [3, 4]: Min { value: 3 }, [1, 2]: Min { value: 1 }, [0, 10]: Min { value: 2 }, [5, 8]: Min { value: 5 }, [1, 4]: Min { value: 1 }, [0, 10]: Min { value: 2 }, [0, 10]: Min { value: 1 }} }";
        assert_eq!(dbg, expected);
    }
}
