use std::range::RangeInclusive;

use bit_vec::BitVec;

use crate::{
    internal_utils::{
        dbg_utils::{as_dbg_tree, lazy_persistent_visitor},
        persistent_utils::PersistentWrapper,
    },
    traits::{LazyNode, Node},
};

/// A persistent segment tree supporting lazy range updates and inclusive range queries.
///
/// Updates can branch from any existing version without changing its logical values.
/// Queries and prefix searches take `&mut self` because they materialize pending
/// updates and may allocate nodes, but they do not create versions or change values.
///
/// It uses `O(n + q*log(n + 1))` arena space, where `q` counts updates,
/// queries, and prefix searches, assuming each node uses `O(1)` space.
/// Reads may also allocate nodes while materializing pending updates.
pub struct LazyPersistent<T> {
    nodes: Vec<PersistentWrapper<T>>,
    roots: Vec<usize>,
    n: usize,
}

impl<T> LazyPersistent<T>
where
    T: LazyNode + Clone,
{
    /// Builds a lazy persistent segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// Takes `O(n)` time, assuming cloning and [`combine`](Node::combine) take constant time.
    /// A nonempty slice creates version `0`; an empty slice creates no versions.
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut temp = Self {
            nodes: Vec::with_capacity(4 * n),
            roots: Vec::with_capacity(1),
            n,
        };
        if n == 0 {
            return temp;
        }
        let root = temp.build_helper(values, 0, n - 1);
        temp.roots.push(root);
        temp
    }

    fn build_helper(&mut self, values: &[T], i: usize, j: usize) -> usize {
        if i == j {
            let curr_node = self.nodes.len();
            self.nodes.push(values[i].clone().into());
            return curr_node;
        }
        let mid = usize::midpoint(i, j);
        let left_node = self.build_helper(values, i, mid);
        let right_node = self.build_helper(values, mid + 1, j);
        let curr_node = self.nodes.len();
        self.nodes.push(Node::combine(
            &self.nodes[left_node],
            &self.nodes[right_node],
        ));
        self.nodes[curr_node].set_children(left_node, right_node);
        curr_node
    }

    /// Returns the aggregate in `version` of the inclusive range `[left, right]`
    /// intersected with the original slice. Returns `None` if the intersection
    /// or tree is empty. Empty trees return `None` regardless of `version`.
    ///
    /// Takes `O(log(n + 1))` time, assuming cloning, [`combine`](Node::combine),
    /// [`defer`](LazyNode::defer), and [`apply_pending`](LazyNode::apply_pending)
    /// take constant time. May allocate nodes while materializing pending updates.
    ///
    /// # Panics
    /// Panics if the tree is nonempty and `version` is not in `0..self.versions()`.
    pub fn query(&mut self, version: usize, left: usize, right: usize) -> Option<T> {
        if self.n == 0 {
            return None;
        }
        self.query_helper(
            self.roots[version],
            RangeInclusive {
                start: left,
                last: right,
            },
            RangeInclusive {
                start: 0,
                last: self.n - 1,
            },
        )
        .map(PersistentWrapper::into_inner)
    }

    // Pending updates are excluded from the aggregate until applied. Children
    // must be cloned before inheriting one so shared versions keep their values.
    fn push(&mut self, curr_node: usize, range: RangeInclusive<usize>) {
        if self.nodes[curr_node].pending().is_none() {
            return;
        }
        if range.start != range.last {
            let left_node = self.nodes.len();
            let right_node = self.nodes.len() + 1;
            self.nodes.push(
                self.nodes[self.nodes[curr_node]
                    .left_child()
                    .unwrap_or_else(|| panic!("{range:?}"))
                    .get()]
                .clone(),
            );
            self.nodes.push(
                self.nodes[self.nodes[curr_node]
                    .right_child()
                    .unwrap_or_else(|| panic!("{range:?}"))
                    .get()]
                .clone(),
            );
            let (parent_slice, sons_slice) = self.nodes.split_at_mut(curr_node + 1);
            let value = parent_slice[curr_node].pending().unwrap();
            sons_slice[left_node - curr_node - 1].defer(value);
            sons_slice[right_node - curr_node - 1].defer(value);
            self.nodes[curr_node].set_children(left_node, right_node);
        }
        self.nodes[curr_node].apply_pending(range);
    }

    fn query_helper(
        &mut self,
        curr_node: usize,
        query_range: RangeInclusive<usize>,
        index_range: RangeInclusive<usize>,
    ) -> Option<PersistentWrapper<T>> {
        if index_range.last < query_range.start || query_range.last < index_range.start {
            return None;
        }
        if self.nodes[curr_node].pending().is_some() {
            self.push(curr_node, index_range);
        }
        if query_range.start <= index_range.start && index_range.last <= query_range.last {
            return Some(self.nodes[curr_node].clone());
        }
        let mid = usize::midpoint(index_range.start, index_range.last);
        let left_node = self.nodes[curr_node].left_child().unwrap().get();
        let right_node = self.nodes[curr_node].right_child().unwrap().get();
        match (
            self.query_helper(
                left_node,
                query_range,
                RangeInclusive {
                    start: index_range.start,
                    last: mid,
                },
            ),
            self.query_helper(
                right_node,
                query_range,
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

    /// Creates a new segment tree version by applying `value` to the inclusive range `[left, right]` in `version`.
    ///
    /// Intersects the inclusive range with the original slice. If the intersection
    /// or tree is empty, does nothing: no version is created and `version` is not checked.
    /// Otherwise, the new version has index `self.versions() - 1` after the call;
    /// existing versions retain their values.
    ///
    /// Takes `O(log(n + 1))` time, assuming cloning, [`combine`](Node::combine),
    /// [`defer`](LazyNode::defer), and [`apply_pending`](LazyNode::apply_pending)
    /// take constant time.
    ///
    /// # Panics
    /// Panics if the intersection is nonempty and `version` is not in `0..self.versions()`.
    pub fn update(
        &mut self,
        version: usize,
        left: usize,
        right: usize,
        value: &<T as LazyNode>::Update,
    ) {
        if self.n == 0 || left > right || left >= self.n {
            return;
        }
        let right = right.min(self.n - 1);
        let new_root = self.update_helper(
            self.roots[version],
            RangeInclusive {
                start: left,
                last: right,
            },
            value,
            RangeInclusive {
                start: 0,
                last: self.n - 1,
            },
        );
        self.roots.push(new_root);
    }

    fn update_helper(
        &mut self,
        curr_node: usize,
        query_range: RangeInclusive<usize>,
        value: &<T as LazyNode>::Update,
        index_range: RangeInclusive<usize>,
    ) -> usize {
        if index_range.last < query_range.start || query_range.last < index_range.start {
            // This sibling may be untouched by the new update, but its aggregate
            // must include any older pending update before the parent combines it.
            self.push(curr_node, index_range);
            return curr_node;
        }
        let x = self.nodes.len();
        self.nodes.push(self.nodes[curr_node].clone());
        if query_range.start <= index_range.start && index_range.last <= query_range.last {
            self.nodes[x].defer(value);
            self.push(x, index_range);
            return x;
        }
        self.push(x, index_range);
        let mid = usize::midpoint(index_range.start, index_range.last);
        let left_node = self.update_helper(
            self.nodes[x].left_child().unwrap().get(),
            query_range,
            value,
            RangeInclusive {
                start: index_range.start,
                last: mid,
            },
        );
        let right_node = self.update_helper(
            self.nodes[x].right_child().unwrap().get(),
            query_range,
            value,
            RangeInclusive {
                start: mid + 1,
                last: index_range.last,
            },
        );
        self.nodes[x] = Node::combine(&self.nodes[left_node], &self.nodes[right_node]);
        self.nodes[x].set_children(left_node, right_node);
        x
    }

    /// Returns the number of versions available.
    ///
    /// Building from a nonempty slice creates one version; an empty slice creates
    /// none. Each successful [`update`](Self::update) adds one version.
    /// Queries and prefix searches do not add versions.
    #[allow(clippy::must_use_candidate)]
    pub const fn versions(&self) -> usize {
        self.roots.len()
    }

    /// Returns the zero-based last index of the smallest nonempty prefix[^note]
    /// `u` in `version` such that `predicate(u.value(), value)` is `true`.
    ///
    /// A matching prefix must exist: this API cannot represent a missing match
    /// and falls through to the last index if no prefix matches.
    /// The following must be true:
    /// - `predicate` is monotonic over prefixes[^note2].
    /// - `g` will satisfy the following, given segments `[i,j]` and `[i,k]` with `j<k` we have that `predicate([i,k].value(),value)` implies `predicate([j+1,k].value(),g([i,j].value(),value))`.
    ///
    /// Takes `O(log(n + 1))` time if cloning, lazy-node operations, `predicate`,
    /// and `g` take constant time. May allocate nodes while materializing pending
    /// updates, without changing any version's logical values.
    ///
    /// # Panics
    /// Panics if `version` is not in `0..self.versions()` (including an empty tree).
    ///
    /// # Examples
    /// The first example finds the smallest prefix whose sum reaches a target.
    /// ```
    /// # use seg_tree::{LazyPersistent,utils::LazySum ,traits::Node};
    /// let predicate = |left_value: &usize, value: &usize|{ *left_value >= *value }; // Is the sum greater or equal to value?
    /// let g = |left_node: &usize, value: usize|{ value - *left_node }; // Subtract the sum of the prefix.
    /// # let nodes: Vec<LazySum<usize>> = (0..10).map(|x| LazySum::initialize(&x)).collect();
    /// let mut seg_tree = LazyPersistent::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with LazySum<usize> nodes
    /// let index = seg_tree.lower_bound(0, predicate, g, 3); // Will return 2 as sum([0,1,2])>=3
    /// # let sums = vec![0,1,3,6,10,15,21,28,36,45];
    /// # for i in 0..10{
    /// #    assert_eq!(seg_tree.lower_bound(0, predicate, g, sums[i]), i);
    /// # }
    /// ```
    /// Searches also respect updates and branches from earlier versions:
    /// ```
    /// use seg_tree::{LazyPersistent, traits::Node, utils::LazySum};
    /// let nodes: Vec<_> = [1usize, 2, 3, 4].iter().map(LazySum::initialize).collect();
    /// let mut tree = LazyPersistent::build(&nodes);
    /// tree.update(0, 0, 3, &2); // Version 1: [3, 4, 5, 6].
    /// tree.update(0, 0, 0, &10); // Version 2 branches from 0: [11, 2, 3, 4].
    /// let predicate = |sum: &usize, target: &usize| sum >= target;
    /// let remaining = |sum: &usize, target: usize| target - sum;
    /// assert_eq!(tree.lower_bound(0, predicate, remaining, 6), 2);
    /// assert_eq!(tree.lower_bound(1, predicate, remaining, 6), 1);
    /// assert_eq!(tree.lower_bound(2, predicate, remaining, 6), 0);
    /// assert_eq!(*tree.query(0, 0, 3).unwrap().value(), 10);
    /// assert_eq!(tree.versions(), 3);
    /// ```
    /// The final example finds the first position whose value is at most a target.
    /// ```
    /// use seg_tree::{LazyPersistent, utils::LazyMin, traits::Node};
    /// let nodes: Vec<LazyMin<usize>> = (0..10).rev().map(|x| LazyMin::initialize(&x)).collect();
    /// let mut tree = LazyPersistent::build(&nodes); // [9,8,7,6,5,4,3,2,1,0].
    /// let predicate = |minimum: &usize, target: &usize| minimum <= target;
    /// let g = |_minimum: &usize, target: usize| target;
    /// assert_eq!(tree.lower_bound(0, predicate, g, 6), 3);
    /// tree.update(0, 0, 9, &4); // Assign every element to 4 in version 1.
    /// assert_eq!(tree.lower_bound(1, predicate, g, 6), 0);
    /// assert_eq!(tree.lower_bound(0, predicate, g, 6), 3);
    /// ```
    ///
    /// [^note]: A prefix is a segment of the form `[0,i]`.
    ///
    /// [^note2]: Given two prefixes `u` and `v` if `u` is contained in `v` then `predicate(u.value(), value)` implies `predicate(v.value(), value)`.
    pub fn lower_bound<F, G>(
        &mut self,
        version: usize,
        predicate: F,
        g: G,
        value: <T as Node>::Value,
    ) -> usize
    where
        F: Fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
        G: Fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    {
        self.lower_bound_helper(
            self.roots[version],
            RangeInclusive {
                start: 0,
                last: self.n - 1,
            },
            predicate,
            g,
            value,
        )
    }
    fn lower_bound_helper<F, G>(
        &mut self,
        curr_node: usize,
        index_range: RangeInclusive<usize>,
        predicate: F,
        g: G,
        value: <T as Node>::Value,
    ) -> usize
    where
        F: Fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
        G: Fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    {
        self.push(curr_node, index_range);
        if index_range.start == index_range.last {
            return index_range.start;
        }
        let mid = usize::midpoint(index_range.start, index_range.last);
        let left_node = self.nodes[curr_node].left_child().unwrap().get();
        let right_node = self.nodes[curr_node].right_child().unwrap().get();
        let left_range = RangeInclusive {
            start: index_range.start,
            last: mid,
        };
        self.push(left_node, left_range);
        let left_value = self.nodes[left_node].value();
        if predicate(left_value, &value) {
            self.lower_bound_helper(left_node, left_range, predicate, g, value)
        } else {
            let value = g(left_value, value);
            self.lower_bound_helper(
                right_node,
                RangeInclusive {
                    start: mid + 1,
                    last: index_range.last,
                },
                predicate,
                g,
                value,
            )
        }
    }
}

impl<T> core::fmt::Debug for LazyPersistent<T>
where
    T: core::fmt::Debug + LazyNode,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = self.nodes.len();
        f.debug_struct("LazyPersistent")
            .field("n", &self.n)
            .field("root_nodes", &self.roots)
            .field(
                "nodes",
                &as_dbg_tree(&self.nodes, {
                    |nodes, f| {
                        let mut visited = BitVec::from_elem(len, false);
                        for root_node in &self.roots {
                            lazy_persistent_visitor(
                                *root_node,
                                0,
                                self.n - 1,
                                f,
                                nodes,
                                &mut visited,
                            );
                        }
                    }
                }),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        internal_utils::test_utils::{assert_prefix_search_matches, assert_range_matches},
        segment_tree::lazy_persistent::LazyPersistent,
        traits::Node,
        utils::LazySum,
    };
    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<LazySum<usize>> = (0..=10).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        assert!(segment_tree.query(0, 0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<LazySum<usize>> = (0..=10).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        assert!(segment_tree.query(0, 10, 0).is_none());
    }
    #[test]
    fn normal_update_works() {
        let nodes: Vec<LazySum<usize>> = (0..=10).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, 0, &value);
        assert_eq!(segment_tree.query(1, 0, 0).unwrap().value(), &value);
    }

    #[test]
    fn branched_update_works() {
        let nodes: Vec<LazySum<usize>> = (0..=10).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, 10, &value);
        segment_tree.update(0, 1, 1, &value);
        assert_eq!(segment_tree.query(2, 0, 0).unwrap().value(), &0);
        assert_eq!(segment_tree.query(2, 1, 1).unwrap().value(), &(value + 1));
    }

    #[test]
    fn query_works() {
        let nodes: Vec<LazySum<usize>> = (0..=10).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        assert_eq!(segment_tree.query(0, 0, 10).unwrap().value(), &55);
    }

    #[test]
    fn leaf_update_works() {
        // Regression test: a range update landing exactly on a single leaf
        // (left == right) used to panic in `push`, since a leaf's
        // `left_child()`/`right_child()` are `None` and `push` wrongly
        // treated any non-`is_empty` range, including a single point, as
        // having children to descend into.
        let nodes: Vec<LazySum<usize>> = (0..=10).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        let value = 20;
        segment_tree.update(0, 3, 3, &value);
        assert_eq!(segment_tree.query(1, 3, 3).unwrap().value(), &(3 + value));
        // Original version and neighboring leaves are untouched.
        assert_eq!(segment_tree.query(0, 3, 3).unwrap().value(), &3);
        assert_eq!(segment_tree.query(1, 2, 2).unwrap().value(), &2);
    }

    #[test]
    fn chained_update_preserves_pending_sibling() {
        let nodes = vec![LazySum::<usize>::initialize(&0); 2];
        let mut segment_tree = LazyPersistent::build(&nodes);
        segment_tree.update(0, 0, 1, &10);
        segment_tree.update(1, 0, 0, &5);

        assert_eq!(segment_tree.query(2, 0, 1).unwrap().value(), &25);
        assert_eq!(segment_tree.query(2, 0, 0).unwrap().value(), &15);
        assert_eq!(segment_tree.query(2, 1, 1).unwrap().value(), &10);
        assert_eq!(segment_tree.query(1, 0, 1).unwrap().value(), &20);
        assert_eq!(segment_tree.query(0, 0, 1).unwrap().value(), &0);
    }

    #[test]
    fn lower_bound_after_whole_range_update_uses_pending_values() {
        let nodes = vec![LazySum::<usize>::initialize(&0); 2];
        let mut segment_tree = LazyPersistent::build(&nodes);
        segment_tree.update(0, 0, 1, &10);

        assert_eq!(
            segment_tree.lower_bound(
                1,
                |sum, target| sum >= target,
                |sum, target| target - sum,
                5
            ),
            0
        );
    }

    #[test]
    fn deeper_updates_preserve_inherited_pending_state() {
        let nodes = vec![LazySum::<usize>::initialize(&0); 8];
        let mut segment_tree = LazyPersistent::build(&nodes);
        segment_tree.update(0, 0, 7, &10);
        segment_tree.update(1, 0, 3, &5);
        segment_tree.update(2, 1, 2, &2);

        let expected = [15, 17, 17, 15, 10, 10, 10, 10];
        assert_range_matches(
            &expected,
            |left, right| *segment_tree.query(3, left, right).unwrap().value(),
            |range| range.iter().sum::<usize>(),
        );
    }

    #[test]
    fn queries_before_chained_update_do_not_change_results() {
        let nodes = vec![LazySum::<usize>::initialize(&0); 4];
        let mut unqueried = LazyPersistent::build(&nodes);
        let mut queried = LazyPersistent::build(&nodes);
        unqueried.update(0, 0, 3, &10);
        queried.update(0, 0, 3, &10);
        for index in (0..4).rev() {
            assert_eq!(queried.query(1, index, index).unwrap().value(), &10);
        }
        unqueried.update(1, 0, 0, &5);
        queried.update(1, 0, 0, &5);

        let unqueried_sum = *unqueried.query(2, 0, 3).unwrap().value();
        let queried_sum = *queried.query(2, 0, 3).unwrap().value();
        assert_eq!(unqueried_sum, queried_sum);
        assert_eq!(unqueried_sum, 45);
    }

    #[test]
    fn lower_bound_is_independent_of_prior_queries() {
        let nodes = vec![LazySum::<usize>::initialize(&0); 8];
        let mut unqueried = LazyPersistent::build(&nodes);
        let mut queried = LazyPersistent::build(&nodes);
        unqueried.update(0, 0, 7, &10);
        queried.update(0, 0, 7, &10);
        for index in (0..8).rev() {
            assert_eq!(queried.query(1, index, index).unwrap().value(), &10);
        }

        for target in [5, 10, 11, 40, 41, 80] {
            let before = unqueried.lower_bound(
                1,
                |sum, target| sum >= target,
                |sum, target| target - sum,
                target,
            );
            let after = queried.lower_bound(
                1,
                |sum, target| sum >= target,
                |sum, target| target - sum,
                target,
            );
            assert_eq!(before, after, "target {target}");
            assert_eq!(before, (target - 1) / 10, "target {target}");
        }
    }

    #[test]
    fn branches_from_updated_version_preserve_all_versions() {
        let nodes = vec![LazySum::<usize>::initialize(&0); 4];
        let mut segment_tree = LazyPersistent::build(&nodes);
        segment_tree.update(0, 0, 3, &10);
        segment_tree.update(1, 0, 1, &5);
        segment_tree.update(1, 2, 3, &7);
        assert_eq!(segment_tree.versions(), 4);

        let expected = [
            [0, 0, 0, 0],
            [10, 10, 10, 10],
            [15, 15, 10, 10],
            [10, 10, 17, 17],
        ];
        // Visit descendants before their ancestors, then repeat in the opposite order.
        for version in [3, 2, 1, 0, 0, 1, 2, 3] {
            assert_range_matches(
                &expected[version],
                |left, right| *segment_tree.query(version, left, right).unwrap().value(),
                |range| range.iter().sum::<usize>(),
            );
        }
        assert_eq!(segment_tree.versions(), 4);
    }

    #[test]
    fn branching_updates_and_searches_match_vector_versions() {
        for n in [1, 2, 3, 5, 8, 9] {
            let initial: Vec<_> = (0..n).map(|i| i % 4 + 1).collect();
            let nodes: Vec<_> = initial.iter().map(LazySum::initialize).collect();
            let mut tree = LazyPersistent::build(&nodes);
            let mut expected = vec![initial];
            let mut seed = n as u64;
            for step in 0..16 {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                let version = (seed % expected.len() as u64) as usize;
                let left = (seed % n as u64) as usize;
                let right = ((seed >> 16) % n as u64) as usize;
                let (left, right) = match step % 4 {
                    0 => (0, n - 1),
                    1 => (left, left),
                    _ => (left.min(right), left.max(right)),
                };
                let delta = (seed % 3 + 1) as usize;
                if step % 2 == 0 {
                    assert_eq!(
                        *tree.query(version, left, left).unwrap().value(),
                        expected[version][left]
                    );
                }
                let mut next = expected[version].clone();
                for value in &mut next[left..=right] {
                    *value += delta;
                }
                tree.update(version, left, right, &delta);
                expected.push(next);
                assert_eq!(tree.versions(), expected.len());
            }
            for _ in 0..2 {
                for (version, values) in expected.iter().enumerate().rev() {
                    let total: usize = values.iter().sum();
                    assert_prefix_search_matches(
                        values,
                        [0, 1, total / 2, total],
                        |target| {
                            Some(tree.lower_bound(
                                version,
                                |sum, target| sum >= target,
                                |sum, target| target - sum,
                                *target,
                            ))
                        },
                        |prefix, target| prefix.iter().sum::<usize>() >= *target,
                    );
                    assert_range_matches(
                        values,
                        |left, right| *tree.query(version, left, right).unwrap().value(),
                        |range| range.iter().sum::<usize>(),
                    );
                }
            }
        }
    }

    #[test]
    fn noncommutative_updates_preserve_chronological_order_and_versions() {
        use crate::traits::LazyNode;

        #[derive(Clone)]
        struct AffineSum {
            sum: usize,
            pending: Option<(usize, usize)>,
        }

        impl Node for AffineSum {
            type Value = usize;

            fn initialize(value: &usize) -> Self {
                Self {
                    sum: *value,
                    pending: None,
                }
            }

            fn combine(a: &Self, b: &Self) -> Self {
                Self::initialize(&(a.sum + b.sum))
            }

            fn value(&self) -> &usize {
                &self.sum
            }
        }

        impl LazyNode for AffineSum {
            type Update = (usize, usize);

            fn pending(&self) -> Option<&Self::Update> {
                self.pending.as_ref()
            }

            fn defer(&mut self, &(mul, add): &Self::Update) {
                self.pending = Some(match self.pending {
                    Some((old_mul, old_add)) => (mul * old_mul, mul * old_add + add),
                    None => (mul, add),
                });
            }

            fn apply_pending(&mut self, range: core::range::RangeInclusive<usize>) {
                if let Some((mul, add)) = self.pending.take() {
                    self.sum = mul * self.sum + add * (range.last - range.start + 1);
                }
            }
        }

        let nodes: Vec<_> = (1..=5).map(|value| AffineSum::initialize(&value)).collect();
        let mut tree = LazyPersistent::build(&nodes);
        tree.update(0, 0, 4, &(1, 3));
        tree.update(1, 1, 3, &(0, 2));
        tree.update(1, 2, 4, &(2, 0));
        tree.update(2, 0, 4, &(2, 1));
        tree.update(4, 2, 4, &(0, 7));
        let expected = [
            vec![1, 2, 3, 4, 5],
            vec![4, 5, 6, 7, 8],
            vec![4, 2, 2, 2, 8],
            vec![4, 5, 12, 14, 16],
            vec![9, 5, 5, 5, 17],
            vec![9, 5, 7, 7, 7],
        ];
        for _ in 0..2 {
            for (version, values) in expected.iter().enumerate().rev() {
                let total: usize = values.iter().sum();
                assert_prefix_search_matches(
                    values,
                    [0, 1, total / 2, total],
                    |target| {
                        Some(tree.lower_bound(
                            version,
                            |sum, target| sum >= target,
                            |sum, target| target - sum,
                            *target,
                        ))
                    },
                    |prefix, target| prefix.iter().sum::<usize>() >= *target,
                );
                assert_range_matches(
                    values,
                    |left, right| *tree.query(version, left, right).unwrap().value(),
                    |range| range.iter().sum::<usize>(),
                );
            }
        }
    }

    #[test]
    #[should_panic]
    fn query_panics_when_version_equals_version_count() {
        let nodes = vec![LazySum::<usize>::initialize(&0); 2];
        let mut segment_tree = LazyPersistent::build(&nodes);
        let invalid_version = segment_tree.versions();
        segment_tree.query(invalid_version, 0, 1);
    }

    #[test]
    #[should_panic]
    fn update_panics_when_version_equals_version_count() {
        let nodes = vec![LazySum::<usize>::initialize(&0); 2];
        let mut segment_tree = LazyPersistent::build(&nodes);
        let invalid_version = segment_tree.versions();
        segment_tree.update(invalid_version, 0, 1, &5);
    }

    #[test]
    fn empty_tree_has_no_versions() {
        let segment_tree = LazyPersistent::<LazySum<usize>>::build(&[]);
        assert_eq!(segment_tree.versions(), 0);
    }

    #[test]
    fn empty_tree_update_does_not_create_a_version() {
        let mut segment_tree = LazyPersistent::<LazySum<usize>>::build(&[]);
        segment_tree.update(0, 0, 0, &5);
        assert_eq!(segment_tree.versions(), 0);
    }

    #[test]
    fn dbg_works() {
        let nodes: Vec<LazySum<usize>> = (0..=10).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        segment_tree.update(0, 0, 1, &2);
        let dbg = format!("{segment_tree:?}");
        let expected = "LazyPersistent { n: 11, root_nodes: [20, 21], nodes: {[0, 10]: LazySum { value: 55, update: None }, [0, 5]: LazySum { value: 15, update: None }, [0, 2]: LazySum { value: 3, update: None }, [0, 1]: LazySum { value: 1, update: None }, [0, 0]: LazySum { value: 0, update: None }, [1, 1]: LazySum { value: 1, update: None }, [2, 2]: LazySum { value: 2, update: None }, [3, 5]: LazySum { value: 12, update: None }, [3, 4]: LazySum { value: 7, update: None }, [3, 3]: LazySum { value: 3, update: None }, [4, 4]: LazySum { value: 4, update: None }, [5, 5]: LazySum { value: 5, update: None }, [6, 10]: LazySum { value: 40, update: None }, [6, 8]: LazySum { value: 21, update: None }, [6, 7]: LazySum { value: 13, update: None }, [6, 6]: LazySum { value: 6, update: None }, [7, 7]: LazySum { value: 7, update: None }, [8, 8]: LazySum { value: 8, update: None }, [9, 10]: LazySum { value: 19, update: None }, [9, 9]: LazySum { value: 9, update: None }, [10, 10]: LazySum { value: 10, update: None }, [0, 10]: LazySum { value: 59, update: None }, [0, 5]: LazySum { value: 19, update: None }, [0, 2]: LazySum { value: 7, update: None }, [0, 1]: LazySum { value: 5, update: None }, [0, 0]: LazySum { value: 0, update: Some(2) }, [1, 1]: LazySum { value: 1, update: Some(2) }} }";
        pretty_assertions::assert_eq!(dbg, expected);
    }
}
