use core::mem::MaybeUninit;
#[cfg(feature = "bifurcate")]
use core::ops::Range;

use crate::{
    internal_utils::dbg_utils::{as_dbg_tree, recursive_visitor},
    traits::Node,
};

/**
Segment tree with range queries and point updates.
It uses `O(n)` space, assuming that each node uses `O(1)` space.
Note if you don't need to use `lower_bound`, just use [`Iterative`](crate::segment_tree::Iterative) it uses half the memory and it's more performant.
*/
pub struct Recursive<T> {
    // Only nodes reachable from the root for [0, n) are initialized.
    nodes: Vec<MaybeUninit<T>>,
    n: usize,
}

impl<T> Recursive<T>
where
    T: Node + Clone,
{
    /// Builds segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// It has time complexity of `O(n*log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    #[must_use]
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        if n == 0 {
            return Self {
                nodes: Vec::new(),
                n: 0,
            };
        }
        let mut nodes = Vec::with_capacity(4 * n);
        // SAFETY: As the nodes are of type `MaybeUninit<T>` this is safe.
        unsafe { nodes.set_len(4 * n) };
        Self::build_helper(0, 0, n - 1, values, &mut nodes);

        Self { nodes, n }
    }

    #[inline]
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

    /// Sets the p-th element of the segment tree to value T and update the segment tree correspondingly.
    /// It will panic if p is not in `[0,n)`
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    pub fn update(&mut self, p: usize, value: &<T as Node>::Value) {
        self.update_helper(p, value, 0, 0, self.n - 1);
    }

    #[inline]
    fn update_helper(
        &mut self,
        p: usize,
        value: &<T as Node>::Value,
        curr_node: usize,
        i: usize,
        j: usize,
    ) {
        if j < p || p < i {
            return;
        }
        if i == j {
            let replacement = Node::initialize(value);
            // SAFETY: This reachable leaf was initialized by build_helper. Assignment
            // installs the replacement even if dropping the previous value panics.
            unsafe { *self.nodes[curr_node].assume_init_mut() = replacement };
            return;
        }
        let mid = usize::midpoint(i, j);
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        self.update_helper(p, value, left_node, i, mid);
        self.update_helper(p, value, right_node, mid + 1, j);
        // SAFETY: Both children are reachable nodes initialized by build_helper.
        let replacement = unsafe {
            Node::combine(
                self.nodes[left_node].assume_init_ref(),
                self.nodes[right_node].assume_init_ref(),
            )
        };
        // SAFETY: This reachable parent is initialized, including after a prior
        // assignment unwound while dropping its old value.
        unsafe { *self.nodes[curr_node].assume_init_mut() = replacement };
    }

    /// Returns the aggregate of the inclusive range `[left, right]` intersected
    /// with the original slice. Returns `None` if the intersection or tree is empty.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    #[allow(clippy::must_use_candidate)]
    pub fn query(&self, left: usize, right: usize) -> Option<T> {
        if self.n == 0 {
            return None;
        }
        self.query_helper(left, right, 0, 0, self.n - 1)
    }

    #[inline]
    fn query_helper(
        &self,
        left: usize,
        right: usize,
        curr_node: usize,
        i: usize,
        j: usize,
    ) -> Option<T> {
        if j < left || right < i {
            return None;
        }
        let mid = usize::midpoint(i, j);
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        if left <= i && j <= right {
            // SAFETY: The query follows only reachable nodes initialized by build_helper.
            return Some(unsafe { self.nodes[curr_node].assume_init_ref() }.clone());
        }
        match (
            self.query_helper(left, right, left_node, i, mid),
            self.query_helper(left, right, right_node, mid + 1, j),
        ) {
            (Some(ans_left), Some(ans_right)) => Some(Node::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }
}

impl<T> Recursive<T> {
    #[inline]
    fn drop_helper(curr_node: usize, i: usize, j: usize, nodes: &mut [MaybeUninit<T>]) {
        if i < j {
            let mid = usize::midpoint(i, j);
            let left_node = 2 * curr_node + 1;
            let right_node = 2 * curr_node + 2;
            Self::drop_helper(left_node, i, mid, nodes);
            Self::drop_helper(right_node, mid + 1, j, nodes);
        }
        // SAFETY: Only reachable, initialized nodes are visited, each exactly once.
        unsafe { nodes[curr_node].assume_init_drop() };
    }
}

impl<T> Drop for Recursive<T> {
    fn drop(&mut self) {
        if self.n == 0 {
            return;
        }
        Self::drop_helper(0, 0, self.n - 1, &mut self.nodes);
    }
}

impl<T> core::fmt::Debug for Recursive<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Recursive")
            .field("n", &self.n)
            .field(
                "nodes",
                &as_dbg_tree(&self.nodes, |nodes, f| {
                    if self.n > 0 {
                        // SAFETY: build_helper initialized every node reachable by this
                        // traversal; unused slots are never visited.
                        unsafe { recursive_visitor(0, 0, self.n - 1, f, nodes) };
                    }
                }),
            )
            .finish()
    }
}

#[cfg(feature = "bifurcate")]
use bifurcate::{Bisectable, MidPoint};

#[cfg(feature = "bifurcate")]
impl<T> Recursive<T>
where
    T: Node + Clone,
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
        T: Node + Clone,
        F: FnMut(&<T as Node>::Value) -> std::cmp::Ordering,
    {
        use std::cmp::Ordering;
        if i == j {
            return Some(i);
        }
        let mid = usize::mid_point(&i, &j)?;
        let left_node_index = 2 * curr_node_index + 1;
        let right_node_index = 2 * curr_node_index + 2;
        // SAFETY: This is the reachable left child of a non-leaf node.
        let left_node = unsafe { self.nodes.get(left_node_index)?.assume_init_ref() };
        let temp_node = carry_node
            .as_ref()
            .map_or_else(|| (*left_node).clone(), |v| T::combine(v, left_node));
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
        T: Node + Clone,
        F: FnMut(&<T as Node>::Value) -> std::cmp::Ordering,
    {
        use std::cmp::Ordering;
        if i == j {
            return Some(i);
        }
        let mid = usize::mid_point(&i, &j)?;
        let left_node_index = 2 * curr_node_index + 1;
        let right_node_index = 2 * curr_node_index + 2;
        // SAFETY: This is the reachable left child of a non-leaf node.
        let left_node = unsafe { self.nodes.get(left_node_index)?.assume_init_ref() };
        let temp_node = carry_node
            .as_ref()
            .map_or_else(|| (*left_node).clone(), |v| T::combine(v, left_node));
        match f(temp_node.value()) {
            Ordering::Less | Ordering::Equal => {
                self.bisect_right_helper(f, right_node_index, mid + 1, j, Some(temp_node))
            }
            Ordering::Greater => self.bisect_right_helper(f, left_node_index, i, mid, carry_node),
        }
    }

    fn equal_range_helper<F>(
        &self,
        mut f: F,
        curr_node_index: usize,
        i: usize,
        j: usize,
        carry_node: Option<T>,
    ) -> Option<Range<usize>>
    where
        T: Node + Clone,
        F: FnMut(&<T as Node>::Value) -> std::cmp::Ordering,
    {
        use std::cmp::Ordering;
        if i == j {
            return Some(i..i);
        }
        let mid = usize::mid_point(&i, &j)?;
        let left_node_index = 2 * curr_node_index + 1;
        let right_node_index = 2 * curr_node_index + 2;
        // SAFETY: This is the reachable left child of a non-leaf node.
        let left_node = unsafe { self.nodes.get(left_node_index)?.assume_init_ref() };
        let temp_node = carry_node
            .as_ref()
            .map_or_else(|| (*left_node).clone(), |v| T::combine(v, left_node));
        match f(temp_node.value()) {
            Ordering::Less => {
                self.equal_range_helper(f, right_node_index, mid + 1, j, Some(temp_node))
            }
            Ordering::Greater => self.equal_range_helper(f, left_node_index, i, mid, carry_node),
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
/// **NOTE**: This implementation searches through prefixes,
/// not through the underlying elements themselves
#[cfg_attr(docsrs, doc(cfg(feature = "bifurcate")))]
impl<T> Bisectable for Recursive<T>
where
    T: Node + Clone,
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

    fn equal_range_by<F>(&self, f: F) -> Option<std::ops::Range<Self::Index>>
    where
        F: FnMut(&Self::Value) -> std::cmp::Ordering,
    {
        (self.n > 0).then_some(())?;
        self.equal_range_helper(f, 0, 0, self.n.checked_sub(1)?, None)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "bifurcate")]
    use crate::internal_utils::test_utils::assert_prefix_search_matches;
    use crate::{internal_utils::test_utils::assert_range_matches, traits::Node, utils::Min};

    use super::Recursive;

    #[cfg(miri)]
    #[derive(Clone)]
    struct DropNode(Box<usize>);

    #[cfg(miri)]
    impl Node for DropNode {
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

    #[cfg(miri)]
    impl Drop for DropNode {
        fn drop(&mut self) {
            // Read the owned allocation so dropping an uninitialized slot cannot
            // be optimized into a no-op.
            std::hint::black_box(*self.0);
        }
    }

    #[test]
    fn drop_empty_tree_does_not_panic() {
        drop(Recursive::<Min<usize>>::build(&[]));
    }

    #[cfg(miri)]
    #[test]
    fn miri_drop_owned_nodes() {
        for len in [0, 1, 5] {
            let nodes: Vec<_> = (0..len).map(|x| DropNode::initialize(&x)).collect();
            let tree = Recursive::build(&nodes);
            // Drop only reachable nodes, leaving uninitialized storage untouched.
            drop(tree);
        }
    }

    #[test]
    fn update_drop_panic_leaves_initialized_nodes() {
        use std::{
            cell::Cell,
            panic::{AssertUnwindSafe, catch_unwind},
            rc::Rc,
        };

        #[derive(Clone, Debug)]
        struct PanicNode {
            value: Box<usize>,
            panic_on: Rc<Cell<Option<usize>>>,
        }

        impl Node for PanicNode {
            type Value = (usize, Rc<Cell<Option<usize>>>);

            fn initialize(value: &Self::Value) -> Self {
                Self {
                    value: Box::new(value.0),
                    panic_on: value.1.clone(),
                }
            }

            fn combine(a: &Self, b: &Self) -> Self {
                Self::initialize(&(*a.value + *b.value, a.panic_on.clone()))
            }

            fn value(&self) -> &Self::Value {
                unreachable!("this regression does not use Node::value")
            }
        }

        impl Drop for PanicNode {
            fn drop(&mut self) {
                if self.panic_on.get() == Some(*self.value) {
                    self.panic_on.set(None);
                    panic!("node destructor panic");
                }
            }
        }

        // Exercise replacement of both a leaf and its parent in a tree with holes.
        for panic_value in [1, 3] {
            let panic_on = Rc::new(Cell::new(None));
            let nodes: Vec<_> = (1..=5)
                .map(|value| PanicNode::initialize(&(value, panic_on.clone())))
                .collect();
            let mut tree = Recursive::build(&nodes);
            panic_on.set(Some(panic_value));
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    tree.update(0, &(10, panic_on.clone()));
                }))
                .is_err()
            );
            assert_eq!(*tree.query(0, 0).unwrap().value, 10);
            assert!(!format!("{tree:?}").is_empty());
            drop(tree);
        }
    }

    #[test]
    fn dbg_empty_tree() {
        assert_eq!(
            format!("{:?}", Recursive::<Min<usize>>::build(&[])),
            "Recursive { n: 0, nodes: {} }"
        );
    }

    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Recursive::build(&nodes);
        assert!(segment_tree.query(0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Recursive::build(&nodes);
        assert!(segment_tree.query(10, 0).is_none());
    }
    #[test]
    fn update_works() {
        let mut values = [8, 3, 7, 1, 6];
        let nodes: Vec<Min<usize>> = values.iter().map(Min::initialize).collect();
        let mut segment_tree = Recursive::build(&nodes);
        for (index, value) in [(0, 20), (4, 0), (2, 2), (4, 9), (0, 1)] {
            segment_tree.update(index, &value);
            values[index] = value;
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
            let nodes: Vec<Min<usize>> = values.iter().map(Min::initialize).collect();
            let segment_tree = Recursive::build(&nodes);
            assert_range_matches(
                values,
                |left, right| *segment_tree.query(left, right).unwrap().value(),
                |range| *range.iter().min().unwrap(),
            );
        }
    }

    #[test]
    fn dbg_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = Recursive::build(&nodes);
        segment_tree.update(0, &2);
        let dbg = format!("{segment_tree:?}");
        let expected = "Recursive { n: 11, nodes: {[0, 10]: Min { value: 1 }, [0, 5]: Min { value: 1 }, [0, 2]: Min { value: 1 }, [0, 1]: Min { value: 1 }, [0, 0]: Min { value: 2 }, [1, 1]: Min { value: 1 }, [2, 2]: Min { value: 2 }, [3, 5]: Min { value: 3 }, [3, 4]: Min { value: 3 }, [3, 3]: Min { value: 3 }, [4, 4]: Min { value: 4 }, [5, 5]: Min { value: 5 }, [6, 10]: Min { value: 6 }, [6, 8]: Min { value: 6 }, [6, 7]: Min { value: 6 }, [6, 6]: Min { value: 6 }, [7, 7]: Min { value: 7 }, [8, 8]: Min { value: 8 }, [9, 10]: Min { value: 9 }, [9, 9]: Min { value: 9 }, [10, 10]: Min { value: 10 }} }";
        pretty_assertions::assert_eq!(dbg, expected);
    }

    #[test]
    #[cfg(feature = "bifurcate")]
    fn simple_bisect_left() {
        use bifurcate::Bisectable;

        use crate::utils::Max;

        let values = [0, 2, 2, 5, 3, 8, 6, 10, 10, 9, 7];
        let nodes: Vec<Max<usize>> = values.iter().map(Max::initialize).collect();
        let segment_tree = Recursive::build(&nodes);
        assert_prefix_search_matches(
            &values,
            0..=10,
            |target| segment_tree.bisect_left(target),
            |prefix, target| prefix.iter().max().unwrap() >= target,
        );
    }

    #[test]
    #[cfg(feature = "bifurcate")]
    fn simple_bisect_right() {
        use bifurcate::Bisectable;

        use crate::utils::Max;

        let nodes: Vec<Max<usize>> = (0..=10).map(|x| Max::initialize(&x)).collect();
        let segment_tree = Recursive::build(&nodes);
        let value = 5;
        let found = segment_tree.bisect_right(&value).unwrap();
        assert_eq!(found, 6);
    }

    #[test]
    #[cfg(feature = "bifurcate")]
    fn simple_equal_range() {
        use bifurcate::Bisectable;

        use crate::utils::Max;

        let nodes: Vec<Max<usize>> = (0..=10).map(|x| Max::initialize(&x)).collect();
        let segment_tree = Recursive::build(&nodes);
        let value = 5;
        let found = segment_tree.equal_range(&value).unwrap();
        assert_eq!(found, 5..6);
    }

    #[test]
    #[cfg(feature = "bifurcate")]
    fn equal_range() {
        use bifurcate::Bisectable;

        use crate::utils::Max;

        let nodes: Vec<Max<usize>> = (0..=20)
            .map(|x| x / 2)
            .map(|x| Max::initialize(&x))
            .collect();
        let segment_tree = Recursive::build(&nodes);
        let value = 5;
        let found = segment_tree.equal_range(&value).unwrap();
        assert_eq!(found, 10..12);
    }
}
