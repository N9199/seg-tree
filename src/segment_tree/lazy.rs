use crate::nodes::{LazyNode, Node};

/// Lazy segment tree with range queries and range updates.
/// It uses `O(n)` space, assuming that each node uses `O(1)` space.
pub struct LazySegmentTree<T: LazyNode> {
    nodes: Vec<T>,
    n: usize,
}

impl<T: LazyNode + Clone> LazySegmentTree<T> {
    /// Builds lazy segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// It has time complexity of `O(n*log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut nodes = Vec::with_capacity(4 * n);
        for _ in 0..4 {
            for v in values {
                nodes.push(v.clone());
            }
        }
        let mut out = Self { nodes, n };
        if n == 0 {
            return out;
        }
        out.build_helper(0, 0, n - 1, values);
        out
    }

    fn build_helper(&mut self, curr_node: usize, i: usize, j: usize, values: &[T]) {
        if i == j {
            self.nodes[curr_node] = values[i].clone();
            return;
        }
        let mid = (i + j) / 2;
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        self.build_helper(left_node, i, mid, values);
        self.build_helper(right_node, mid + 1, j, values);
        self.nodes[curr_node] = T::combine(&self.nodes[left_node], &self.nodes[right_node]);
    }

    fn push(&mut self, u: usize, i: usize, j: usize) {
        // parent_slice.len() == u + 1 && sons_slice.len() == 4*self.n - (u + 1)
        let (parent_slice, sons_slice) = self.nodes.split_at_mut(u + 1);
        if let Some(value) = parent_slice[u].lazy_value() {
            if i != j {
                sons_slice[u].update_lazy_value(value); // At 2*u + 1 - (u + 1)
                sons_slice[u + 1].update_lazy_value(value); // At 2*u + 2 - (u + 1)
            }
        }
        self.nodes[u].lazy_update(i, j);
    }

    /// Updates the range `[i,j]` with value.
    /// It will panic if `i` or `j` is not in `[0,n]`.
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine), [update_lazy_value](LazyNode::update_lazy_value) and [lazy_update](LazyNode::lazy_update) have constant time complexity.
    pub fn update(&mut self, i: usize, j: usize, value: <T as Node>::Value) {
        self.update_helper(i, j, &value, 0, 0, self.n - 1);
    }

    fn update_helper(
        &mut self,
        left: usize,
        right: usize,
        value: &<T as Node>::Value,
        curr_node: usize,
        i: usize,
        j: usize,
    ) {
        if self.nodes[curr_node].lazy_value().is_some() {
            self.push(curr_node, i, j);
        }
        if j < left || right < i {
            return;
        }
        if left <= i && j <= right {
            self.nodes[curr_node].update_lazy_value(value);
            self.push(curr_node, i, j);
            return;
        }
        let mid = (i + j) / 2;
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        self.update_helper(left, right, value, left_node, i, mid);
        self.update_helper(left, right, value, right_node, mid + 1, j);
        self.nodes[curr_node] = T::combine(&self.nodes[left_node], &self.nodes[right_node]);
    }

    /// Returns the result from the range `[left,right]`.
    /// It returns None if and only if range is empty.
    /// It will **panic** if `left` or `right` are not in [0,n).
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine), [update_lazy_value](LazyNode::update_lazy_value) and [lazy_update](LazyNode::lazy_update) have constant time complexity.
    pub fn query(&mut self, left: usize, right: usize) -> Option<T> {
        self.query_helper(left, right, 0, 0, self.n - 1)
    }

    fn query_helper(
        &mut self,
        left: usize,
        right: usize,
        curr_node: usize,
        i: usize,
        j: usize,
    ) -> Option<T> {
        if j < left || right < i {
            return None;
        }
        let mid = (i + j) / 2;
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        if self.nodes[curr_node].lazy_value().is_some() {
            self.push(curr_node, i, j);
        }
        if left <= i && j <= right {
            return Some(self.nodes[curr_node].clone());
        }
        match (
            self.query_helper(left, right, left_node, i, mid),
            self.query_helper(left, right, right_node, mid + 1, j),
        ) {
            (Some(ans_left), Some(ans_right)) => Some(T::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }

    /// A method that finds the smallest prefix[^note] `u` such that `predicate(u.value(), value)` is `true`. The following must be true:
    /// - `predicate` is monotonic over prefixes[^note2].
    /// - `g` will satisfy the following, given segments `[i,j]` and `[i,k]` with `j<k` we have that `predicate([i,k].value(),value)` implies `predicate([j+1,k].value(),g([i,j].value(),value))`.
    ///
    /// These are two examples, the first is finding the smallest prefix which sums at least some value.
    /// ```
    /// # use seg_tree::{segment_tree::LazySegmentTree,utils::Sum,nodes::Node};
    /// let predicate = |left_value:&usize, value:&usize|{*left_value>=*value}; // Is the sum greater or equal to value?
    /// let g = |left_node:&usize,value:usize|{value-*left_node}; // Subtract the sum of the prefix.
    /// # let nodes: Vec<Sum<usize>> = (0..10).map(|x| Sum::initialize(&x)).collect();
    /// let seg_tree = LazySegmentTree::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with Sum<usize> nodes
    /// let index = seg_tree.lower_bound(predicate, g, 3); // Will return 2 as sum([0,1,2])>=3
    /// # let sums = vec![0,1,3,6,10,15,21,28,36,45];
    /// # for i in 0..10{
    /// #    assert_eq!(seg_tree.lower_bound(predicate, g, sums[i]), i);
    /// # }
    /// ```
    /// The second is finding the position of the smallest value greater or equal to some value.
    /// ```
<<<<<<< HEAD
    /// # use seg_tree::{segment_tree::LazySegmentTree,utils::{Max,LazySetWrapper},nodes::Node};
    /// # type LSMax<T> = LazySetWrapper<Max<T>>;
=======
<<<<<<< HEAD
    /// # use seg_tree::{segment_tree::LazySegmentTree,default::Max,nodes::Node};
=======
    /// # use seg_tree::{segment_tree::LazySegmentTree,utils::{Max,LazySetWrapper},nodes::Node};
    /// # type LSMax<T> = LazySetWrapper<Max<T>>;
>>>>>>> 019bb64 (Fix doctests on lazy segment trees)
>>>>>>> 82028da (Fix doctests on lazy segment trees)
    /// let predicate = |left_value:&usize, value:&usize|{*left_value>=*value}; // Is the maximum greater or equal to value?
    /// let g = |_left_node:&usize,value:usize|{value}; // Do nothing
    /// # let nodes: Vec<LSMax<usize>> = (0..10).map(|x| LSMax::initialize(&x)).collect();
    /// let seg_tree = LazySegmentTree::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with Max<usize> nodes
    /// let index = seg_tree.lower_bound(predicate, g, 3); // Will return 3 as 3>=3
    /// # for i in 0..10{
    /// #    assert_eq!(seg_tree.lower_bound(predicate, g, i), i);
    /// # }
    /// ```
    ///
    /// [^note]: A prefix is a segment of the form `[0,i]`.
    ///
    /// [^note2]: Given two prefixes `u` and `v` if `u` is contained in `v` then `predicate(u.value(), value)` implies `predicate(v.value(), value)`.
    pub fn lower_bound<F, G>(&self, predicate: F, g: G, value: <T as Node>::Value) -> usize
    where
        F: Fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
        G: Fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    {
        self.lower_bound_helper(0, 0, self.n - 1, predicate, g, value)
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
        let mid = (i + j) / 2;
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        let left_value = self.nodes[left_node].value();
        if predicate(left_value, &value) {
            self.lower_bound_helper(left_node, i, mid, predicate, g, value)
        } else {
            let value = g(left_value, value);
            self.lower_bound_helper(right_node, mid + 1, j, predicate, g, value)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        nodes::Node,
        utils::{LazySetWrapper, Min},
    };

    use super::LazySegmentTree;
    // TODO Add more tests

    type LSMin<T> = LazySetWrapper<Min<T>>;

    #[test]
    fn build_works() {
        let n = 16;
        let nodes: Vec<LSMin<usize>> = (0..n).map(|x| LSMin::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        for i in 0..n {
            let temp = segment_tree.query(i, i).unwrap();
            assert_eq!(temp.value(), &i);
        }
    }
    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<LSMin<usize>> = (0..10).map(|x| LSMin::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 9).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<LSMin<usize>> = (0..10).map(|x| LSMin::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        assert!(segment_tree.query(10, 0).is_none());
    }
    #[test]
    fn update_works() {
        let nodes: Vec<LSMin<usize>> = (0..10).map(|x| LSMin::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        let value = 20;
        segment_tree.update(0, 9, value);
        assert_eq!(segment_tree.query(0, 1).unwrap().value(), &value);
    }
    #[test]
    fn query_works() {
        let nodes: Vec<LSMin<usize>> = (0..10).map(|x| LSMin::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        assert_eq!(segment_tree.query(1, 9).unwrap().value(), &1);
    }
}
