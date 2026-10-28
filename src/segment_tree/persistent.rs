use bit_vec::BitVec;

use crate::{
    internal_utils::{
        dbg_utils::{as_dbg_tree, persistent_visitor},
        persistent_utils::PersistentWrapper,
    },
    traits::Node,
};

/// Persistent segment tree, it saves every version of itself, it has range queries and point updates.
///
/// It uses `O(n+q*log(n))` space, where `q` is the amount of updates, and assuming that each node uses `O(1)` space.
pub struct Persistent<T> {
    nodes: Vec<PersistentWrapper<T>>,
    roots: Vec<usize>,
    n: usize,
}

impl<T> Persistent<T>
where
    T: Clone + Node,
{
    /// Builds persistent segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// It has time complexity of `O(n*log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
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
        let mid = usize::midpoint(i, j);
        if i == j {
            let curr_node = self.nodes.len();
            self.nodes.push(values[i].clone().into());
            return curr_node;
        }
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
    /// # Panics
    /// Panics if the tree is nonempty and `version` is not in `0..self.versions()`.
    ///
    /// # Complexity
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    #[allow(clippy::must_use_candidate)]
    pub fn query(&self, version: usize, left: usize, right: usize) -> Option<T> {
        if self.n == 0 {
            return None;
        }
        self.query_helper(self.roots[version], left, right, 0, self.n - 1)
            .map(PersistentWrapper::into_inner)
    }

    fn query_helper(
        &self,
        curr_node: usize,
        left: usize,
        right: usize,
        i: usize,
        j: usize,
    ) -> Option<PersistentWrapper<T>> {
        if j < left || right < i {
            return None;
        }
        if left <= i && j <= right {
            return Some(self.nodes[curr_node].clone());
        }
        let mid = usize::midpoint(i, j);
        let left_node = self.nodes[curr_node].left_child().unwrap().get();
        let right_node = self.nodes[curr_node].right_child().unwrap().get();
        match (
            self.query_helper(left_node, left, right, i, mid),
            self.query_helper(right_node, left, right, mid + 1, j),
        ) {
            (Some(ans_left), Some(ans_right)) => Some(Node::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }

    /// Creates a new segment tree version from version were the p-th element of the segment tree to value T and update the segment tree correspondingly.
    /// Does nothing if `p` is outside the original slice, including an empty tree;
    /// in that case no version is created and `version` is not checked.
    /// Otherwise, panics if `version` is not in `0..self.versions()`.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine) has constant time complexity.
    pub fn update(&mut self, version: usize, p: usize, value: &<T as Node>::Value) {
        if p >= self.n {
            return;
        }
        let new_root = self.update_helper(self.roots[version], p, value, 0, self.n - 1);
        self.roots.push(new_root);
    }

    fn update_helper(
        &mut self,
        curr_node: usize,
        p: usize,
        value: &<T as Node>::Value,
        i: usize,
        j: usize,
    ) -> usize {
        if j < p || p < i {
            return curr_node;
        }
        let x = self.nodes.len();
        self.nodes.push(self.nodes[curr_node].clone());
        if i == j {
            self.nodes[x] = Node::initialize(value);
            return x;
        }
        let mid = usize::midpoint(i, j);
        let left_node =
            self.update_helper(self.nodes[x].left_child().unwrap().get(), p, value, i, mid);
        let right_node = self.update_helper(
            self.nodes[x].right_child().unwrap().get(),
            p,
            value,
            mid + 1,
            j,
        );
        self.nodes[x] = Node::combine(&self.nodes[left_node], &self.nodes[right_node]);
        self.nodes[x].set_children(left_node, right_node);
        x
    }
    /// Returns the amount of different versions the current segment tree has. Essentially this will be how many calls to [`update`](Self::update) have happened.
    #[allow(clippy::must_use_candidate)]
    pub const fn versions(&self) -> usize {
        self.roots.len()
    }

    /// A method that finds the smallest prefix[^note] `u` such that `predicate(u.value(), value)` is `true`. The following must be true:
    /// - `predicate` is monotonic over prefixes[^note2].
    /// - `g` will satisfy the following, given segments `[i,j]` and `[i,k]` with `j<k` we have that `predicate([i,k].value(),value)` implies `predicate([j+1,k].value(),g([i,j].value(),value))`.
    ///
    /// These are two examples, the first is finding the smallest prefix which sums at least some value.
    /// ```
    /// # use seg_tree::{Persistent,utils::Sum,traits::Node};
    /// let predicate = |left_value:&usize, value:&usize|{*left_value>=*value}; // Is the sum greater or equal to value?
    /// let g = |left_node:&usize,value:usize|{value-*left_node}; // Subtract the sum of the prefix.
    /// # let nodes: Vec<Sum<usize>> = (0..10).map(|x| Sum::initialize(&x)).collect();
    /// let seg_tree = Persistent::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with Sum<usize> nodes
    /// let index = seg_tree.lower_bound(0, predicate, g, 3); // Will return 2 as sum([0,1,2])>=3
    /// # let sums = vec![0,1,3,6,10,15,21,28,36,45];
    /// # for i in 0..10{
    /// #    assert_eq!(seg_tree.lower_bound(0, predicate, g, sums[i]), i);
    /// # }
    /// ```
    /// The second is finding the position of the smallest value greater or equal to some value.
    /// ```
    /// # use seg_tree::{Persistent,utils::Max, traits::Node};
    /// let predicate = |left_value:&usize, value:&usize|{*left_value>=*value}; // Is the maximum greater or equal to value?
    /// let g = |_left_node:&usize,value:usize|{value}; // Do nothing
    /// # let nodes: Vec<Max<usize>> = (0..10).map(|x| Max::initialize(&x)).collect();
    /// let seg_tree = Persistent::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with Max<usize> nodes
    /// let index = seg_tree.lower_bound(0, predicate, g, 3); // Will return 3 as 3>=3
    /// # for i in 0..10{
    /// #    assert_eq!(seg_tree.lower_bound(0, predicate, g, i), i);
    /// # }
    /// ```
    ///
    /// [^note]: A prefix is a segment of the form `[0,i]`.
    ///
    /// [^note2]: Given two prefixes `u` and `v` if `u` is contained in `v` then `predicate(u.value(), value)` implies `predicate(v.value(), value)`.
    pub fn lower_bound<F, G>(
        &self,
        version: usize,
        predicate: F,
        g: G,
        value: <T as Node>::Value,
    ) -> usize
    where
        F: Fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
        G: Fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    {
        self.lower_bound_helper(self.roots[version], 0, self.n - 1, predicate, g, value)
    }
    fn lower_bound_helper<F, G>(
        &self,
        curr_node: usize,
        i: usize,
        j: usize,
        predicate: F,
        g: G,
        value: <T as Node>::Value,
    ) -> usize
    where
        F: Fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
        G: Fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    {
        if i == j {
            return i;
        }
        let mid = usize::midpoint(i, j);
        let left_node = self.nodes[curr_node].left_child().unwrap().get();
        let right_node = self.nodes[curr_node].right_child().unwrap().get();
        let left_value = self.nodes[left_node].value();
        if predicate(left_value, &value) {
            self.lower_bound_helper(left_node, i, mid, predicate, g, value)
        } else {
            let value = g(left_value, value);
            self.lower_bound_helper(right_node, mid + 1, j, predicate, g, value)
        }
    }
}

impl<T> core::fmt::Debug for Persistent<T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = self.nodes.len();
        f.debug_struct("Persistent")
            .field("n", &self.n)
            .field(
                "nodes",
                &as_dbg_tree(&self.nodes, {
                    |nodes, f| {
                        let mut visited = BitVec::from_elem(len, false);
                        for root_node in &self.roots {
                            persistent_visitor(*root_node, 0, self.n - 1, f, nodes, &mut visited);
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
        segment_tree::Persistent,
        traits::Node,
        utils::Sum,
    };

    #[test]
    fn update_out_of_bounds_does_not_create_a_version() {
        let nodes: Vec<Sum<usize>> = (0..3).map(|x| Sum::initialize(&x)).collect();
        let mut tree = Persistent::build(&nodes);
        tree.update(0, 3, &9);
        assert_eq!(tree.versions(), 1);
        assert_eq!(*tree.query(0, 0, 2).unwrap().value(), 3);
    }

    #[test]
    #[should_panic]
    fn query_invalid_version_panics() {
        let tree = Persistent::build(&[Sum::initialize(&7usize)]);
        tree.query(tree.versions(), 0, 0);
    }

    #[test]
    #[should_panic]
    fn update_invalid_version_panics() {
        let mut tree = Persistent::build(&[Sum::initialize(&7usize)]);
        tree.update(tree.versions(), 0, &9);
    }

    #[test]
    fn empty_tree_debug_does_not_panic() {
        let tree = Persistent::<Sum<usize>>::build(&[]);
        assert_eq!(format!("{tree:?}"), "Persistent { n: 0, nodes: {} }");
    }

    #[test]
    fn singleton_and_non_power_of_two_preserve_versions() {
        for values in [vec![7usize], vec![8, 3, 9, 2, 6]] {
            let nodes: Vec<_> = values.iter().map(Sum::initialize).collect();
            let mut tree = Persistent::build(&nodes);
            assert_eq!(tree.versions(), 1);
            let last = values.len() - 1;
            tree.update(0, last, &11);
            tree.update(0, 0, &13);
            assert_eq!(tree.versions(), 3);
            let mut updated = values.clone();
            updated[last] = 11;
            let mut branched = values.clone();
            branched[0] = 13;
            for (version, expected) in [values, updated, branched].iter().enumerate() {
                for left in 0..expected.len() {
                    for right in left..expected.len() {
                        assert_eq!(
                            *tree.query(version, left, right).unwrap().value(),
                            expected[left..=right].iter().sum::<usize>()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let segment_tree = Persistent::build(&nodes);
        assert!(segment_tree.query(0, 0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let segment_tree = Persistent::build(&nodes);
        assert!(segment_tree.query(0, 10, 0).is_none());
    }
    #[test]
    fn normal_update_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = Persistent::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, &value);
        let mut updated_values: Vec<usize> = (0..=10).collect();
        updated_values[0] = value;
        for (version, values) in [(0, (0..=10).collect()), (1, updated_values)] {
            assert_range_matches(
                &values,
                |left, right| *segment_tree.query(version, left, right).unwrap().value(),
                |range| range.iter().sum::<usize>(),
            );
        }
    }

    #[test]
    fn branched_update_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = Persistent::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, &value);
        segment_tree.update(0, 1, &value);
        let original: Vec<usize> = (0..=10).collect();
        let mut first_branch = original.clone();
        first_branch[0] = value;
        let mut second_branch = original.clone();
        second_branch[1] = value;
        for (version, values) in [original, first_branch, second_branch].iter().enumerate() {
            assert_range_matches(
                values,
                |left, right| *segment_tree.query(version, left, right).unwrap().value(),
                |range| range.iter().sum::<usize>(),
            );
        }
    }

    #[test]
    fn query_works() {
        for values in [vec![4usize], vec![0, 3, 1], (0..=10).collect()] {
            let nodes: Vec<_> = values.iter().map(Sum::initialize).collect();
            let segment_tree = Persistent::build(&nodes);
            assert_range_matches(
                &values,
                |left, right| *segment_tree.query(0, left, right).unwrap().value(),
                |range| range.iter().sum::<usize>(),
            );
        }
    }

    #[test]
    fn prefix_search_matches_across_updated_and_branched_versions() {
        for values in [
            vec![0usize],
            vec![0, 3, 1],
            vec![2, 0, 4, 0, 1],
            vec![0, 2, 1, 0, 3, 0, 4, 1],
        ] {
            let nodes: Vec<_> = values.iter().map(Sum::initialize).collect();
            let mut tree = Persistent::build(&nodes);
            let mut versions = vec![values];
            let last = versions[0].len() - 1;
            for (parent, index, value) in [(0, 0, 5), (1, last, 0), (0, last, 7), (3, 0, 0)] {
                tree.update(parent, index, &value);
                let mut updated = versions[parent].clone();
                updated[index] = value;
                versions.push(updated);
            }
            assert_eq!(tree.versions(), versions.len());
            for (version, values) in versions.iter().enumerate() {
                assert_range_matches(
                    values,
                    |left, right| *tree.query(version, left, right).unwrap().value(),
                    |range| range.iter().sum::<usize>(),
                );
                assert_prefix_search_matches(
                    values,
                    0..=values.iter().sum::<usize>(),
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
            }
        }
    }

    #[test]
    fn dbg_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = Persistent::build(&nodes);
        segment_tree.update(0, 1, &2);
        let dbg = format!("{segment_tree:?}");
        let expected = "Persistent { n: 11, nodes: {[0, 10]: Sum { value: 55 }, [0, 5]: Sum { value: 15 }, [0, 2]: Sum { value: 3 }, [0, 1]: Sum { value: 1 }, [0, 0]: Sum { value: 0 }, [1, 1]: Sum { value: 1 }, [2, 2]: Sum { value: 2 }, [3, 5]: Sum { value: 12 }, [3, 4]: Sum { value: 7 }, [3, 3]: Sum { value: 3 }, [4, 4]: Sum { value: 4 }, [5, 5]: Sum { value: 5 }, [6, 10]: Sum { value: 40 }, [6, 8]: Sum { value: 21 }, [6, 7]: Sum { value: 13 }, [6, 6]: Sum { value: 6 }, [7, 7]: Sum { value: 7 }, [8, 8]: Sum { value: 8 }, [9, 10]: Sum { value: 19 }, [9, 9]: Sum { value: 9 }, [10, 10]: Sum { value: 10 }, [0, 10]: Sum { value: 56 }, [0, 5]: Sum { value: 16 }, [0, 2]: Sum { value: 4 }, [0, 1]: Sum { value: 2 }, [1, 1]: Sum { value: 2 }} }";
        pretty_assertions::assert_eq!(dbg, expected);
    }
}
