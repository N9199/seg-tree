use std::range::RangeInclusive;

use bit_vec::BitVec;

use crate::{
    internal_utils::{
        dbg_utils::{as_dbg_tree, lazy_persistent_visitor},
        persistent_utils::PersistentWrapper,
    },
    traits::{LazyNode, Node},
};

/// Lazy persistent segment tree, it saves every version of itself, it has range queries and range updates.
///
/// It uses `O(n+q*log(n))` space, where `q` is the amount of updates, and assuming that each node uses `O(1)` space.
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

    /// Returns the result from the range `[left,right]` from the version of the segment tree.
    /// It returns None if and only if range is empty.
    /// It will **panic** if left or right are not in `[0,n)`, or if version is not in <code>[0,[versions](Self::versions))</code>.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine), [`update_update`](LazyNode::update_update) and [`lazy_update`](LazyNode::lazy_update) have constant time complexity.
    pub fn query(&mut self, version: usize, left: usize, right: usize) -> Option<T> {
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

    fn push(&mut self, curr_node: usize, range: RangeInclusive<usize>) {
        if self.nodes[curr_node].pending().is_some() && !range.is_empty() {
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

    /// Creates a new segment tree version from version were the p-th element of the segment tree to value T and update the segment tree correspondingly.
    // It will panic if p is not in `[0,n)`, or if version is not in <code>[0,[versions](Self::versions))</code>
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine), [`update_update`](LazyNode::update_update) and [`lazy_update`](LazyNode::lazy_update) have constant time complexity.
    pub fn update(
        &mut self,
        version: usize,
        left: usize,
        right: usize,
        value: &<T as LazyNode>::Update,
    ) {
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
            return curr_node;
        }
        let x = self.nodes.len();
        self.nodes.push(self.nodes[curr_node].clone());
        if query_range.start <= index_range.start && index_range.last <= query_range.last {
            self.nodes[x].defer(value);
            self.push(x, index_range);
            return x;
        }
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
    /// The second is finding the position of the smallest value greater or equal to some value.
    /// ```
    /// # use seg_tree::{LazyPersistent,utils::{Max, LazySetWrapper},traits::Node};
    /// # type PMax<T> = LazySetWrapper<Max<T>>;
    /// let predicate = |left_value:&usize, value:&usize|{*left_value>=*value}; // Is the maximum greater or equal to value?
    /// let g = |_left_node:&usize,value:usize|{value}; // Do nothing
    /// # let nodes: Vec<PMax<usize>> = (0..10).map(|x| PMax::initialize(&x)).collect();
    /// let mut seg_tree = LazyPersistent::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with Max<usize> nodes
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
        if index_range.start == index_range.last {
            return index_range.start;
        }
        let mid = usize::midpoint(index_range.start, index_range.last);
        let left_node = if let Some(l) = self.nodes[curr_node].left_child() {
            l.get()
        } else {
            self.push(curr_node, index_range);
            self.nodes[curr_node].left_child().unwrap().get()
        };
        let right_node = self.nodes[curr_node].right_child().unwrap().get();
        let left_value = self.nodes[left_node].value();
        if predicate(left_value, &value) {
            self.lower_bound_helper(
                left_node,
                RangeInclusive {
                    start: index_range.start,
                    last: mid,
                },
                predicate,
                g,
                value,
            )
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
    use crate::{segment_tree::lazy_persistent::LazyPersistent, traits::Node, utils::LazySum};
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
    fn dbg_works() {
        let nodes: Vec<LazySum<usize>> = (0..=10).map(|x| LazySum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        segment_tree.update(0, 0, 1, &2);
        let dbg = format!("{segment_tree:?}");
        let expected = "LazyPersistent { n: 11, root_nodes: [20, 21], nodes: {[0, 10]: LazySum { value: 55, update: None }, [0, 5]: LazySum { value: 15, update: None }, [0, 2]: LazySum { value: 3, update: None }, [0, 1]: LazySum { value: 1, update: None }, [0, 0]: LazySum { value: 0, update: None }, [1, 1]: LazySum { value: 1, update: None }, [2, 2]: LazySum { value: 2, update: None }, [3, 5]: LazySum { value: 12, update: None }, [3, 4]: LazySum { value: 7, update: None }, [3, 3]: LazySum { value: 3, update: None }, [4, 4]: LazySum { value: 4, update: None }, [5, 5]: LazySum { value: 5, update: None }, [6, 10]: LazySum { value: 40, update: None }, [6, 8]: LazySum { value: 21, update: None }, [6, 7]: LazySum { value: 13, update: None }, [6, 6]: LazySum { value: 6, update: None }, [7, 7]: LazySum { value: 7, update: None }, [8, 8]: LazySum { value: 8, update: None }, [9, 10]: LazySum { value: 19, update: None }, [9, 9]: LazySum { value: 9, update: None }, [10, 10]: LazySum { value: 10, update: None }, [0, 10]: LazySum { value: 59, update: None }, [0, 5]: LazySum { value: 19, update: None }, [0, 2]: LazySum { value: 7, update: None }, [0, 1]: LazySum { value: 5, update: None }, [0, 0]: LazySum { value: 0, update: Some(2) }, [1, 1]: LazySum { value: 1, update: Some(2) }} }";
        pretty_assertions::assert_eq!(dbg, expected);
    }
}
