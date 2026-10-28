use core::mem::MaybeUninit;
#[cfg(feature = "bifurcate")]
use core::ops::Range;
use std::{range::RangeInclusive, sync::Mutex};

use crate::{
    internal_utils::dbg_utils::{as_dbg_tree, recursive_visitor},
    traits::{LazyNode, Node},
};

/// Lazy segment tree with range queries and range updates.
/// It uses `O(n)` space, assuming that each node uses `O(1)` space.
pub struct LazyRecursive<T> {
    // Only nodes reachable from the root for [0, n) are initialized.
    nodes: Mutex<Vec<MaybeUninit<T>>>,
    n: usize,
}

impl<T> LazyRecursive<T>
where
    T: LazyNode + Clone,
{
    /// Builds lazy segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// It has time complexity of `O(n*log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        if n == 0 {
            return Self {
                nodes: Mutex::new(Vec::new()),
                n,
            };
        }
        let mut nodes = Vec::with_capacity(4 * n);
        // SAFETY: As the nodes are of type `MaybeUninit<T>` this is safe.
        unsafe { nodes.set_len(4 * n) };
        Self::build_helper(0, 0, n - 1, values, &mut nodes);
        let nodes = Mutex::new(nodes);
        Self { nodes, n }
    }

    fn build_helper(
        curr_node: usize,
        i: usize,
        j: usize,
        values: &[T],
        nodes: &mut [MaybeUninit<T>],
    ) {
        if i == j {
            nodes[curr_node].write(values[i].clone());
            return;
        }
        let mid = usize::midpoint(i, j);
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        Self::build_helper(left_node, i, mid, values, nodes);
        Self::build_helper(right_node, mid + 1, j, values, nodes);
        let (top_nodes, bottom_nodes) = nodes.split_at_mut(curr_node + 1);
        top_nodes[curr_node].write(Node::combine(
            unsafe { bottom_nodes[left_node - curr_node - 1].assume_init_ref() },
            unsafe { bottom_nodes[right_node - curr_node - 1].assume_init_ref() },
        ));
    }

    fn push(&self, u: usize, range: RangeInclusive<usize>) {
        let mut nodes = self
            .nodes
            .try_lock()
            .expect("Mutex is poisoned, thread should crash");
        debug_assert!(
            u + 1 < nodes.len(),
            "u: {u:?}, nodes.len(): {:?}, range: {range:?}",
            nodes.len()
        );
        let (parent_slice, sons_slice) = nodes.split_at_mut(u + 1);
        // SAFETY: Traversal reaches only nodes initialized by build_helper.
        let parent = unsafe { parent_slice[u].assume_init_mut() };
        if let Some(value) = parent.pending()
            && range.start != range.last
        {
            // SAFETY: A non-leaf's two children are initialized and disjoint from its parent.
            unsafe {
                sons_slice[u].assume_init_mut().defer(value);
                sons_slice[u + 1].assume_init_mut().defer(value);
            }
        }
        parent.apply_pending(range);
    }

    /// Updates the range `[i,j]` with value.
    /// Intersects the inclusive range with the original slice. Does nothing if
    /// the intersection or tree is empty.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine), [`defer`](LazyNode::defer) and [`apply_pending`](LazyNode::apply_pending) have constant time complexity.
    pub fn update(&mut self, i: usize, j: usize, value: &<T as LazyNode>::Update) {
        if self.n == 0 || i > j || i >= self.n {
            return;
        }
        let j = j.min(self.n - 1);
        self.update_helper(
            RangeInclusive { start: i, last: j },
            value,
            0,
            RangeInclusive {
                start: 0,
                last: self.n - 1,
            },
        );
    }

    fn update_helper(
        &mut self,
        query_range: RangeInclusive<usize>,
        value: &<T as LazyNode>::Update,
        curr_node: usize,
        index_range: RangeInclusive<usize>,
    ) {
        self.push(curr_node, index_range);
        if index_range.last < query_range.start || query_range.last < index_range.start {
            return;
        }
        if query_range.start <= index_range.start && index_range.last <= query_range.last {
            // SAFETY: This reachable node is initialized.
            unsafe {
                self.nodes
                    .get_mut()
                    .expect("Mutex is poisoned, thread should crash")[curr_node]
                    .assume_init_mut()
            }
            .defer(value);
            self.push(curr_node, index_range);
            return;
        }
        let mid = usize::midpoint(index_range.start, index_range.last);
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        self.update_helper(
            query_range,
            value,
            left_node,
            RangeInclusive {
                start: index_range.start,
                last: mid,
            },
        );
        self.update_helper(
            query_range,
            value,
            right_node,
            RangeInclusive {
                start: mid + 1,
                last: index_range.last,
            },
        );

        let nodes = self
            .nodes
            .get_mut()
            .expect("Mutex is poisoned, thread should crash");
        // SAFETY: Both children and their parent are reachable, initialized nodes.
        let replacement = unsafe {
            Node::combine(
                nodes[left_node].assume_init_ref(),
                nodes[right_node].assume_init_ref(),
            )
        };
        // Assignment keeps the slot initialized even if dropping its old value panics.
        unsafe { *nodes[curr_node].assume_init_mut() = replacement };
    }

    /// Returns the aggregate of the inclusive range `[left, right]` intersected
    /// with the original slice. Returns `None` if the intersection or tree is empty.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine), [`defer`](LazyNode::defer) and [`apply_pending`](LazyNode::apply_pending) have constant time complexity.
    pub fn query(&self, left: usize, right: usize) -> Option<T> {
        if self.n == 0 {
            return None;
        }
        self.query_helper(
            RangeInclusive {
                start: left,
                last: right,
            },
            0,
            RangeInclusive {
                start: 0,
                last: self.n - 1,
            },
        )
    }

    fn query_helper(
        &self,
        query_range: RangeInclusive<usize>,
        curr_node: usize,
        index_range: RangeInclusive<usize>,
    ) -> Option<T> {
        if index_range.last < query_range.start || query_range.last < index_range.start {
            return None;
        }
        let mid = usize::midpoint(index_range.start, index_range.last);
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        self.push(curr_node, index_range);

        if query_range.start <= index_range.start && index_range.last <= query_range.last {
            let nodes = self
                .nodes
                .lock()
                .expect("Mutex is poisoned, thread should crash");
            // SAFETY: The query follows only reachable, initialized nodes.
            return Some(unsafe { nodes[curr_node].assume_init_ref() }.clone());
        }
        match (
            self.query_helper(
                query_range,
                left_node,
                RangeInclusive {
                    start: index_range.start,
                    last: mid,
                },
            ),
            self.query_helper(
                query_range,
                right_node,
                RangeInclusive {
                    start: mid + 1,
                    last: index_range.last,
                },
            ),
        ) {
            (Some(ans_left), Some(ans_right)) => Some(Node::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }
}

impl<T> LazyRecursive<T> {
    #[inline]
    fn drop_helper(curr_node: usize, i: usize, j: usize, nodes: &mut [MaybeUninit<T>]) {
        if i < j {
            let mid = usize::midpoint(i, j);
            let left_node = 2 * curr_node + 1;
            let right_node = 2 * curr_node + 2;
            Self::drop_helper(left_node, i, mid, nodes);
            Self::drop_helper(right_node, mid + 1, j, nodes);
        }
        // SAFETY: Only initialized nodes are visited, each exactly once.
        unsafe { nodes[curr_node].assume_init_drop() };
    }
}

impl<T> Drop for LazyRecursive<T> {
    fn drop(&mut self) {
        if self.n == 0 {
            return;
        }
        let n = self.n;
        let nodes = self
            .nodes
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::drop_helper(0, 0, n - 1, nodes);
    }
}

#[cfg(feature = "bifurcate")]
use bifurcate::{Bisectable, MidPoint};

#[cfg(feature = "bifurcate")]
impl<T> LazyRecursive<T>
where
    T: LazyNode + Clone,
{
    fn bisect_left_helper<F>(
        &self,
        mut f: F,
        curr_node_index: usize,
        i: usize,
        j: usize,
        carry_node: Option<T>,
    ) -> Option<usize>
    where
        F: FnMut(&<T as Node>::Value) -> std::cmp::Ordering,
    {
        use std::cmp::Ordering;
        if i == j {
            return Some(i);
        }
        let mid = usize::mid_point(&i, &j)?;
        let left_node_index = 2 * curr_node_index + 1;
        let right_node_index = 2 * curr_node_index + 2;
        // We need to check if our node needs to update
        // and then check if our left node needs to update, as we'll combine it later
        self.push(curr_node_index, RangeInclusive { start: i, last: j });
        self.push(
            left_node_index,
            RangeInclusive {
                start: i,
                last: mid,
            },
        );
        let node_lock = self
            .nodes
            .lock()
            .expect("Mutex is poisoned, thread should crash");
        // SAFETY: This reachable child is initialized, including after push.
        let left_node = unsafe { node_lock.get(left_node_index)?.assume_init_ref() };
        let temp_node = carry_node
            .as_ref()
            .map_or_else(|| (*left_node).clone(), |v| T::combine(v, left_node));
        // We don't need the lock anymore, so we'll drop it explicitly right now
        drop(node_lock);

        match f(temp_node.value()) {
            Ordering::Less => {
                self.bisect_left_helper(f, right_node_index, mid + 1, j, Some(temp_node))
            }
            Ordering::Equal | Ordering::Greater => {
                self.bisect_left_helper(f, left_node_index, i, mid, carry_node)
            }
        }
    }
    fn bisect_right_helper<F>(
        &self,
        mut f: F,
        curr_node_index: usize,
        i: usize,
        j: usize,
        carry_node: Option<T>,
    ) -> Option<usize>
    where
        F: FnMut(&<T as Node>::Value) -> std::cmp::Ordering,
    {
        use std::cmp::Ordering;
        if i == j {
            return Some(i);
        }
        let mid = usize::mid_point(&i, &j)?;
        let left_node_index = 2 * curr_node_index + 1;
        let right_node_index = 2 * curr_node_index + 2;
        // We need to check if our node needs to update
        // and then check if our left node needs to update, as we'll combine it later
        self.push(curr_node_index, RangeInclusive { start: i, last: j });
        self.push(
            left_node_index,
            RangeInclusive {
                start: i,
                last: mid,
            },
        );
        let node_lock = self
            .nodes
            .lock()
            .expect("Mutex is poisoned, thread should crash");
        // SAFETY: This reachable child is initialized, including after push.
        let left_node = unsafe { node_lock.get(left_node_index)?.assume_init_ref() };
        let temp_node = carry_node
            .as_ref()
            .map_or_else(|| (*left_node).clone(), |v| T::combine(v, left_node));
        // We don't need the lock anymore, so we'll drop it explicitly right now
        drop(node_lock);

        match f(temp_node.value()) {
            Ordering::Less | Ordering::Equal => {
                self.bisect_right_helper(f, right_node_index, mid + 1, j, Some(temp_node))
            }
            Ordering::Greater => self.bisect_right_helper(f, left_node_index, i, mid, carry_node),
        }
    }
    fn bisect_equal_range_helper<F>(
        &self,
        mut f: F,
        curr_node_index: usize,
        i: usize,
        j: usize,
        carry_node: Option<T>,
    ) -> Option<Range<usize>>
    where
        F: FnMut(&<T as Node>::Value) -> std::cmp::Ordering,
    {
        use std::cmp::Ordering;
        if i == j {
            return Some(i..i);
        }
        let mid = usize::mid_point(&i, &j)?;
        let left_node_index = 2 * curr_node_index + 1;
        let right_node_index = 2 * curr_node_index + 2;
        // We need to check if our node needs to update
        // and then check if our left node needs to update, as we'll combine it later
        self.push(curr_node_index, RangeInclusive { start: i, last: j });
        self.push(
            left_node_index,
            RangeInclusive {
                start: i,
                last: mid,
            },
        );
        let node_lock = self
            .nodes
            .lock()
            .expect("Mutex is poisoned, thread should crash");
        // SAFETY: This reachable child is initialized, including after push.
        let left_node = unsafe { node_lock.get(left_node_index)?.assume_init_ref() };
        let temp_node = carry_node
            .as_ref()
            .map_or_else(|| (*left_node).clone(), |v| T::combine(v, left_node));
        // We don't need the lock anymore, so we'll drop it explicitly right now
        drop(node_lock);

        match f(temp_node.value()) {
            Ordering::Less => {
                self.bisect_equal_range_helper(f, right_node_index, mid + 1, j, Some(temp_node))
            }
            Ordering::Greater => {
                self.bisect_equal_range_helper(f, left_node_index, i, mid, carry_node)
            }
            Ordering::Equal => {
                let left_index =
                    self.bisect_left_helper(&mut f, left_node_index, i, mid, carry_node.clone())?;
                let right_index =
                    self.bisect_right_helper(&mut f, right_node_index, mid + 1, j, carry_node)?;
                Some(left_index..right_index)
            }
        }
    }
}

#[cfg(feature = "bifurcate")]
#[cfg_attr(docsrs, doc(cfg(feature = "bifurcate")))]
impl<T> Bisectable for LazyRecursive<T>
where
    T: LazyNode + Clone,
{
    type Value = <T as Node>::Value;

    type Index = usize;

    fn bisect_left_by<F>(&self, f: F) -> Option<Self::Index>
    where
        F: FnMut(&Self::Value) -> std::cmp::Ordering,
    {
        (self.n > 0).then_some(())?;
        self.bisect_left_helper(f, 0, 0, self.n.checked_sub(1)?, None)
    }

    fn bisect_right_by<F>(&self, f: F) -> Option<Self::Index>
    where
        F: FnMut(&Self::Value) -> std::cmp::Ordering,
    {
        (self.n > 0).then_some(())?;
        self.bisect_right_helper(f, 0, 0, self.n.checked_sub(1)?, None)
    }

    fn equal_range_by<F>(&self, f: F) -> Option<Range<Self::Index>>
    where
        F: FnMut(&Self::Value) -> std::cmp::Ordering,
    {
        (self.n > 0).then_some(())?;
        self.bisect_equal_range_helper(f, 0, 0, self.n.checked_sub(1)?, None)
    }
}

impl<T> core::fmt::Debug for LazyRecursive<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LazyRecursive")
            .field("n", &self.n)
            .field(
                "nodes",
                &as_dbg_tree(
                    &self
                        .nodes
                        .lock()
                        .expect("Mutex is poisoned, thread should crash"),
                    |nodes, f| {
                        if self.n > 0 {
                            // SAFETY: Build initializes every node reached by this traversal.
                            unsafe { recursive_visitor(0, 0, self.n - 1, f, nodes) };
                        }
                    },
                ),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "bifurcate")]
    use crate::internal_utils::test_utils::assert_prefix_search_matches;
    use crate::{
        internal_utils::test_utils::assert_range_matches, traits::Node, utils::LazyMin as LSMin,
    };

    use super::LazyRecursive;

    #[cfg(miri)]
    #[derive(Clone)]
    struct DropNode {
        value: Box<usize>,
        pending: Option<usize>,
    }

    #[cfg(miri)]
    impl Node for DropNode {
        type Value = usize;

        fn initialize(value: &usize) -> Self {
            Self {
                value: Box::new(*value),
                pending: None,
            }
        }

        fn combine(a: &Self, b: &Self) -> Self {
            Self::initialize(&(*a.value + *b.value))
        }

        fn value(&self) -> &usize {
            &self.value
        }
    }

    #[cfg(miri)]
    impl crate::traits::LazyNode for DropNode {
        type Update = usize;

        fn pending(&self) -> Option<&usize> {
            self.pending.as_ref()
        }

        fn defer(&mut self, update: &usize) {
            *self.pending.get_or_insert(0) += update;
        }

        fn apply_pending(&mut self, range: core::range::RangeInclusive<usize>) {
            if let Some(update) = self.pending.take() {
                *self.value += update * (range.last - range.start + 1);
            }
        }
    }

    #[cfg(miri)]
    impl Drop for DropNode {
        fn drop(&mut self) {
            std::hint::black_box(*self.value);
        }
    }

    #[test]
    fn drop_empty_tree_does_not_panic() {
        drop(LazyRecursive::<LSMin<usize>>::build(&[]));
    }

    #[cfg(miri)]
    #[test]
    fn miri_drop_owned_nodes() {
        for len in [0, 1, 5] {
            let nodes: Vec<_> = (0..len).map(|x| DropNode::initialize(&x)).collect();
            let tree = LazyRecursive::build(&nodes);
            // Destroy reachable initialized nodes, leaving unused slots untouched.
            drop(tree);
        }
    }

    #[cfg(miri)]
    #[test]
    fn miri_drop_owned_nodes_after_mutex_poisoning() {
        let tree = LazyRecursive::build(&[DropNode::initialize(&1)]);
        // A panicking update can leave the node-storage mutex in this state.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = tree.nodes.lock().unwrap();
            panic!("poison node storage for the destruction regression");
        }));
        assert!(result.is_err());
        assert!(tree.nodes.is_poisoned());
        drop(tree);
    }

    #[test]
    fn update_right_out_of_bounds_is_normalized() {
        let nodes: Vec<LSMin<usize>> = (0..3).map(|x| LSMin::initialize(&x)).collect();
        let mut tree = LazyRecursive::build(&nodes);
        tree.update(0, 3, &9);
        assert_eq!(*tree.query(0, 2).unwrap().value(), 9);
    }

    #[test]
    fn update_left_out_of_bounds_does_nothing() {
        let nodes: Vec<LSMin<usize>> = (0..3).map(|x| LSMin::initialize(&x)).collect();
        let mut tree = LazyRecursive::build(&nodes);
        tree.update(3, usize::MAX, &9);
        assert_eq!(*tree.query(0, 2).unwrap().value(), 0);
    }

    #[test]
    fn empty_tree_debug_does_not_panic() {
        let tree = LazyRecursive::<LSMin<usize>>::build(&[]);
        assert_eq!(format!("{tree:?}"), "LazyRecursive { n: 0, nodes: {} }");
    }

    #[test]
    fn singleton_and_non_power_of_two_match_slice() {
        for mut values in [vec![7usize], vec![8, 3, 9, 2, 6]] {
            let nodes: Vec<LSMin<usize>> = values.iter().map(LSMin::initialize).collect();
            let mut tree = LazyRecursive::build(&nodes);
            for replacement in [None, Some(11)] {
                if let Some(value) = replacement {
                    let last = values.len() - 1;
                    let first = last / 2;
                    values[first..=last].fill(value);
                    tree.update(first, last, &value);
                }
                for left in 0..values.len() {
                    for right in left..values.len() {
                        assert_eq!(
                            tree.query(left, right).unwrap().value(),
                            values[left..=right].iter().min().unwrap()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn build_works() {
        let n = 16;
        let nodes: Vec<LSMin<usize>> = (0..n).map(|x| LSMin::initialize(&x)).collect();
        let segment_tree = LazyRecursive::build(&nodes);
        let vals: Vec<_> = (0..n)
            .filter_map(|i| {
                let val = segment_tree.query(i, i).unwrap();
                (val.value() != &i).then_some((*val.value(), i))
            })
            .collect();
        assert!(vals.is_empty(), "vals: {vals:?}");
    }
    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<LSMin<usize>> = (0..10).map(|x| LSMin::initialize(&x)).collect();
        let segment_tree = LazyRecursive::build(&nodes);
        assert!(segment_tree.query(0, 9).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<LSMin<usize>> = (0..10).map(|x| LSMin::initialize(&x)).collect();
        let segment_tree = LazyRecursive::build(&nodes);
        assert!(segment_tree.query(10, 0).is_none());
    }
    #[test]
    fn update_works() {
        let mut values = [8, 3, 7, 1, 6];
        let nodes: Vec<LSMin<usize>> = values.iter().map(LSMin::initialize).collect();
        let mut segment_tree = LazyRecursive::build(&nodes);
        for (left, right, value) in [(0, 4, 20), (1, 3, 2), (2, 4, 9), (0, 0, 1), (4, 4, 0)] {
            segment_tree.update(left, right, &value);
            values[left..=right].fill(value);
            assert_range_matches(
                &values,
                |left, right| *segment_tree.query(left, right).unwrap().value(),
                |range| *range.iter().min().unwrap(),
            );
        }
    }
    #[test]
    fn query_works() {
        for values in [
            &[4][..],
            &[8, 3, 7, 1, 6][..],
            &[5, 2, 2, 9, 1, 8, 4, 0][..],
        ] {
            let nodes: Vec<LSMin<usize>> = values.iter().map(LSMin::initialize).collect();
            let segment_tree = LazyRecursive::build(&nodes);
            assert_range_matches(
                values,
                |left, right| *segment_tree.query(left, right).unwrap().value(),
                |range| *range.iter().min().unwrap(),
            );
        }
    }

    #[test]
    fn leaf_update_works() {
        // Regression test: a range update landing exactly on a single leaf
        // (i == j) used to panic in `push`. `!range.is_empty()` is true for
        // a single-point range even though a leaf has no children to push
        // into, since `build_helper` never calls `set_children` on one.
        let nodes: Vec<LSMin<usize>> = (0..10).map(|x| LSMin::initialize(&x)).collect();
        let mut segment_tree = LazyRecursive::build(&nodes);
        let value = 20;
        segment_tree.update(3, 3, &value);
        assert_eq!(segment_tree.query(3, 3).unwrap().value(), &value);
        assert_eq!(segment_tree.query(2, 2).unwrap().value(), &2);
        assert_eq!(segment_tree.query(4, 4).unwrap().value(), &4);
    }

    #[test]
    fn dbg_works() {
        let nodes: Vec<LSMin<usize>> = (0..=10).map(|x| LSMin::initialize(&x)).collect();
        let mut segment_tree = LazyRecursive::build(&nodes);
        segment_tree.update(0, 1, &2);
        let dbg = format!("{segment_tree:?}");
        let expected = "LazyRecursive { n: 11, nodes: {[0, 10]: LazyMin { value: 2, update: None }, [0, 5]: LazyMin { value: 2, update: None }, [0, 2]: LazyMin { value: 2, update: None }, [0, 1]: LazyMin { value: 2, update: None }, [0, 0]: LazyMin { value: 0, update: Some(2) }, [1, 1]: LazyMin { value: 1, update: Some(2) }, [2, 2]: LazyMin { value: 2, update: None }, [3, 5]: LazyMin { value: 3, update: None }, [3, 4]: LazyMin { value: 3, update: None }, [3, 3]: LazyMin { value: 3, update: None }, [4, 4]: LazyMin { value: 4, update: None }, [5, 5]: LazyMin { value: 5, update: None }, [6, 10]: LazyMin { value: 6, update: None }, [6, 8]: LazyMin { value: 6, update: None }, [6, 7]: LazyMin { value: 6, update: None }, [6, 6]: LazyMin { value: 6, update: None }, [7, 7]: LazyMin { value: 7, update: None }, [8, 8]: LazyMin { value: 8, update: None }, [9, 10]: LazyMin { value: 9, update: None }, [9, 9]: LazyMin { value: 9, update: None }, [10, 10]: LazyMin { value: 10, update: None }} }";
        pretty_assertions::assert_eq!(dbg, expected);
    }

    #[test]
    fn compose_accumulates_across_two_updates() {
        // Two overlapping range-add updates should accumulate on any node that
        // still holds unflushed pending state from the first update when the
        // second one arrives, not silently drop or overwrite it.
        use crate::utils::LazySum;
        let nodes: Vec<LazySum<usize>> = (0..8).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyRecursive::build(&nodes);
        segment_tree.update(0, 7, &10); // whole range: +10 everywhere, pushed
        // down and left unflushed partway
        segment_tree.update(0, 3, &5); // sub-range: +5, lands on a node that
        // may still carry unflushed +10
        let values: Vec<usize> = (0..8).map(|x| x + 10 + if x < 4 { 5 } else { 0 }).collect();
        assert_range_matches(
            &values,
            |left, right| *segment_tree.query(left, right).unwrap().value(),
            |range| range.iter().sum::<usize>(),
        );
    }

    #[test]
    #[cfg(feature = "bifurcate")]
    fn bisect_left_after_lazy_updates() {
        use bifurcate::Bisectable;

        use crate::utils::LazySum;

        let mut values = [1, 3, 2, 4, 1];
        let nodes: Vec<LazySum<usize>> = values.iter().map(LazySum::initialize).collect();
        let mut segment_tree = LazyRecursive::build(&nodes);
        assert_prefix_search_matches(
            &values,
            0..=values.iter().sum::<usize>(),
            |target| segment_tree.bisect_left(target),
            |prefix, target| prefix.iter().sum::<usize>() >= *target,
        );
        for (left, right, delta) in [(0, 4, 2), (1, 3, 3), (4, 4, 1)] {
            segment_tree.update(left, right, &delta);
            for value in &mut values[left..=right] {
                *value += delta;
            }
            assert_prefix_search_matches(
                &values,
                0..=values.iter().sum::<usize>(),
                |target| segment_tree.bisect_left(target),
                |prefix, target| prefix.iter().sum::<usize>() >= *target,
            );
        }
    }
}
