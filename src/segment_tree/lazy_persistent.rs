use bit_vec::BitVec;

use crate::{
    internal_utils::{
        dbg_utils::{as_dbg_tree, lazy_persistent_visitor},
        persistent_utils::PersistentWrapper,
    },
    nodes::{LazyNode, Node},
};

/// Lazy persistent segment tree, it saves every version of itself, it has range queries and range updates.
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
        let mid = (i + j) / 2;
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
    /// It will **panic** if left or right are not in `[0,n)`, or if version is not in `[0,`[`versions`](Self::versions)`)`.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine), [`update_lazy_value`](LazyNode::update_lazy_value) and [`lazy_update`](LazyNode::lazy_update) have constant time complexity.
    pub fn query(&mut self, version: usize, left: usize, right: usize) -> Option<T> {
        self.query_helper(self.roots[version], left, right, 0, self.n - 1)
            .map(PersistentWrapper::into_inner)
    }

    fn push(&mut self, curr_node: usize, i: usize, j: usize) {
        if self.nodes[curr_node].lazy_value().is_some() && i != j {
            let left_node = self.nodes.len();
            let right_node = self.nodes.len() + 1;
            self.nodes.push(
                self.nodes[self.nodes[curr_node]
                    .left_child()
                    .unwrap_or_else(|| panic!("[{i}, {j}]"))
                    .get()]
                .clone(),
            );
            self.nodes.push(
                self.nodes[self.nodes[curr_node]
                    .right_child()
                    .unwrap_or_else(|| panic!("[{i}, {j}]"))
                    .get()]
                .clone(),
            );
            let (parent_slice, sons_slice) = self.nodes.split_at_mut(curr_node + 1);
            let value = parent_slice[curr_node].lazy_value().unwrap();
            sons_slice[left_node - curr_node - 1].update_lazy_value(value);
            sons_slice[right_node - curr_node - 1].update_lazy_value(value);
            self.nodes[curr_node].set_children(left_node, right_node);
        }
        self.nodes[curr_node].lazy_update(i, j);
    }

    fn query_helper(
        &mut self,
        curr_node: usize,
        left: usize,
        right: usize,
        i: usize,
        j: usize,
    ) -> Option<PersistentWrapper<T>> {
        if j < left || right < i {
            return None;
        }
        if self.nodes[curr_node].lazy_value().is_some() {
            self.push(curr_node, i, j);
        }
        if left <= i && j <= right {
            return Some(self.nodes[curr_node].clone());
        }
        let mid = (i + j) / 2;
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
    /// It will panic if p is not in `[0,n)`, or if version is not in `[0,`[`versions`](Self::versions)`)`.
    /// It has time complexity of `O(log(n))`, assuming that [`combine`](Node::combine), [`update_lazy_value`](LazyNode::update_lazy_value) and [`lazy_update`](LazyNode::lazy_update) have constant time complexity.
    pub fn update(
        &mut self,
        version: usize,
        left: usize,
        right: usize,
        value: &<T as Node>::Value,
    ) {
        let new_root = self.update_helper(self.roots[version], left, right, value, 0, self.n - 1);
        self.roots.push(new_root);
    }

    fn update_helper(
        &mut self,
        curr_node: usize,
        left: usize,
        right: usize,
        value: &<T as Node>::Value,
        i: usize,
        j: usize,
    ) -> usize {
        if j < left || right < i {
            return curr_node;
        }
        let x = self.nodes.len();
        self.nodes.push(self.nodes[curr_node].clone());
        if left <= i && j <= right {
            self.nodes[x].update_lazy_value(value);
            self.push(x, i, j);
            return x;
        }
        let mid = (i + j) / 2;
        let left_node = self.update_helper(
            self.nodes[x].left_child().unwrap().get(),
            left,
            right,
            value,
            i,
            mid,
        );
        let right_node = self.update_helper(
            self.nodes[x].right_child().unwrap().get(),
            left,
            right,
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
    pub fn versions(&self) -> usize {
        self.roots.len()
    }

    /// A method that finds the smallest prefix[^note] `u` such that `predicate(u.value(), value)` is `true`. The following must be true:
    /// - `predicate` is monotonic over prefixes[^note2].
    /// - `g` will satisfy the following, given segments `[i,j]` and `[i,k]` with `j<k` we have that `predicate([i,k].value(),value)` implies `predicate([j+1,k].value(),g([i,j].value(),value))`.
    ///
    /// These are two examples, the first is finding the smallest prefix which sums at least some value.
    /// ```
    /// # use seg_tree::{LazyPersistent,utils::Sum ,nodes::Node};
    /// let predicate = |left_value: &usize, value: &usize|{ *left_value >= *value }; // Is the sum greater or equal to value?
    /// let g = |left_node: &usize, value: usize|{ value - *left_node }; // Subtract the sum of the prefix.
    /// # let nodes: Vec<Sum<usize>> = (0..10).map(|x| Sum::initialize(&x)).collect();
    /// let mut seg_tree = LazyPersistent::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with Sum<usize> nodes
    /// let index = seg_tree.lower_bound(0, predicate, g, 3); // Will return 2 as sum([0,1,2])>=3
    /// # let sums = vec![0,1,3,6,10,15,21,28,36,45];
    /// # for i in 0..10{
    /// #    assert_eq!(seg_tree.lower_bound(0, predicate, g, sums[i]), i);
    /// # }
    /// ```
    /// The second is finding the position of the smallest value greater or equal to some value.
    /// ```
    /// # use seg_tree::{LazyPersistent,utils::{Max, LazySetWrapper},nodes::Node};
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
        self.lower_bound_helper(self.roots[version], 0, self.n - 1, predicate, g, value)
    }
    fn lower_bound_helper<F, G>(
        &mut self,
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
        let mid = (i + j) / 2;
        let left_node = if let Some(l) = self.nodes[curr_node].left_child() {
            l.get()
        } else {
            self.push(curr_node, i, j);
            self.nodes[curr_node].left_child().unwrap().get()
        };
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
    use crate::{nodes::Node, segment_tree::lazy_persistent::LazyPersistent, utils::Sum};
    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        assert!(segment_tree.query(0, 0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        assert!(segment_tree.query(0, 10, 0).is_none());
    }
    #[test]
    fn normal_update_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, 0, &value);
        assert_eq!(segment_tree.query(1, 0, 0).unwrap().value(), &value);
    }

    #[test]
    fn branched_update_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, 10, &value);
        segment_tree.update(0, 1, 1, &value);
        assert_eq!(segment_tree.query(2, 0, 0).unwrap().value(), &0);
        assert_eq!(segment_tree.query(2, 1, 1).unwrap().value(), &(value + 1));
    }

    #[test]
    fn query_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        assert_eq!(segment_tree.query(0, 0, 10).unwrap().value(), &55);
    }

    #[test]
    fn dbg_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistent::build(&nodes);
        segment_tree.update(0, 0, 1, &2);
        let dbg = format!("{segment_tree:?}");
        let expected = "LazyPersistent { n: 11, root_nodes: [20, 21], nodes: {[0, 10]: Sum { value: 55, lazy_value: None }, [0, 5]: Sum { value: 15, lazy_value: None }, [0, 2]: Sum { value: 3, lazy_value: None }, [0, 1]: Sum { value: 1, lazy_value: None }, [0, 0]: Sum { value: 0, lazy_value: None }, [1, 1]: Sum { value: 1, lazy_value: None }, [2, 2]: Sum { value: 2, lazy_value: None }, [3, 5]: Sum { value: 12, lazy_value: None }, [3, 4]: Sum { value: 7, lazy_value: None }, [3, 3]: Sum { value: 3, lazy_value: None }, [4, 4]: Sum { value: 4, lazy_value: None }, [5, 5]: Sum { value: 5, lazy_value: None }, [6, 10]: Sum { value: 40, lazy_value: None }, [6, 8]: Sum { value: 21, lazy_value: None }, [6, 7]: Sum { value: 13, lazy_value: None }, [6, 6]: Sum { value: 6, lazy_value: None }, [7, 7]: Sum { value: 7, lazy_value: None }, [8, 8]: Sum { value: 8, lazy_value: None }, [9, 10]: Sum { value: 19, lazy_value: None }, [9, 9]: Sum { value: 9, lazy_value: None }, [10, 10]: Sum { value: 10, lazy_value: None }, [0, 10]: Sum { value: 59, lazy_value: None }, [0, 5]: Sum { value: 19, lazy_value: None }, [0, 2]: Sum { value: 7, lazy_value: None }, [0, 1]: Sum { value: 5, lazy_value: None }, [0, 0]: Sum { value: 0, lazy_value: Some(2) }, [1, 1]: Sum { value: 1, lazy_value: Some(2) }} }";
        assert_eq!(dbg, expected);
    }
}
