use crate::nodes::{LazyNode, Node};

/// Lazy segment tree with range queries and range updates.
/// It uses `O(n)` space, assuming that each node uses `O(1)` space.
pub struct LazySegmentTree<T: LazyNode> {
    nodes: Vec<T>,
    n: usize,
}

impl<T: LazyNode + Clone + std::fmt::Debug> LazySegmentTree<T> {
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
        out.build_helper(0, 0, n - 1, values);
        out
    }

    fn build_helper(&mut self, curr_node: usize, i: usize, j: usize, values: &[T]) {
        if i == j {
            self.nodes[curr_node] = values[i].clone();
            println!("{curr_node} [{i},{j}] value: {:?}", self.nodes[curr_node]);
            return;
        }
        let mid = (i + j) / 2;
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        self.build_helper(left_node, i, mid, values);
        self.build_helper(right_node, mid + 1, j, values);
        self.nodes[curr_node] = T::combine(&self.nodes[left_node], &self.nodes[right_node]);
        println!("{curr_node} [{i},{j}] value: {:?}", self.nodes[curr_node]);
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

    /// Updates the range \[i,j\] with value.
    /// It will panic if `i` or `j` is not in \[0,n).
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine), [update_lazy_value](LazyNode::update_lazy_value) and [update_lazy_value](LazyNode::lazy_update) have constant time complexity.
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
        self.nodes[curr_node] = T::combine(&self.nodes[left], &self.nodes[right]);
    }

    /// Returns the result from the range \[left,right\].
    /// It returns None if and only if range is empty.
    /// It will **panic** if `left` or `right` are not in [0,n).
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine), [update_lazy_value](LazyNode::update_lazy_value) and [update_lazy_value](LazyNode::lazy_update) have constant time complexity.
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

    // /// A method that finds the leftmost leaf node `u` such that `predicate(u.value, value)` is `true`.
    // /// `predicate(u.value, value)` must be non-decreasing over the tree, more specifically for every node `u` with left child `v` and right child `w`, if `predicate(v.value, value)` then `predicate(u.value, value)`.
    // /// `g` is used to calculate the new value for a recursive call, more specifically, given a node `u` with corresponding interval \[i,j\], and children `v` amd `w`,
    // /// ```
    // /// # use seg_tree::{segment_tree::LazySegmentTree,default::Sum,nodes::Node};
    // /// let g = |left_node:&usize,value:usize|{value-*left_node};
    // /// let predicate = |left_value:&usize, value:&usize|{*left_value>=*value};
    // /// # let nodes: Vec<Sum<usize>> = (0..10).map(|x| Sum::initialize(&x)).collect();
    // /// let seg_tree = LazySegmentTree::build(&nodes);
    // /// #assert_eq!(seg_tree.lower_bound(predicate, g, 3), 2);
    // /// ```
    // pub fn lower_bound(
    //     &self,
    //     predicate: fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
    //     g: fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    //     value: <T as Node>::Value,
    // ) -> usize {
    //     self.lower_bound_helper(0, 0, self.n - 1, predicate, g, value)
    // }
    // fn lower_bound_helper(
    //     &self,
    //     curr_node: usize,
    //     i: usize,
    //     j: usize,
    //     predicate: fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
    //     g: fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    //     value: <T as Node>::Value,
    // ) -> usize {
    //     if i == j {
    //         return i;
    //     }
    //     let mid = (i + j) / 2;
    //     let left_node = 2 * curr_node + 1;
    //     let right_node = 2 * curr_node + 2;
    //     let left_value = self.nodes[left_node].value();
    //     if predicate(left_value, &value) {
    //         self.lower_bound_helper(left_node, i, mid, predicate, g, value)
    //     } else {
    //         let value = g(left_value, value);
    //         self.lower_bound_helper(right_node, mid + 1, j, predicate, g, value)
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use crate::{default::Min, nodes::Node};

    use super::LazySegmentTree;
    // TODO Add more tests

    #[test]
    fn build_works() {
        let n = 16;
        let nodes: Vec<Min<usize>> = (0..n).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        for i in 0..n {
            let temp = segment_tree.query(i,i).unwrap();
            assert_eq!(temp.value(), &i);
        }
    }
    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Min<usize>> = (0..10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 9).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Min<usize>> = (0..10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        assert!(segment_tree.query(10, 0).is_none());
    }
    #[test]
    fn update_works() {
        let nodes: Vec<Min<usize>> = (0..10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        let value = 20;
        segment_tree.update(0, 9, value);
        assert_eq!(segment_tree.query(0, 1).unwrap().value(), &value);
    }
    #[test]
    fn query_works() {
        let nodes: Vec<Min<usize>> = (0..10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        assert_eq!(segment_tree.query(1, 9).unwrap().value(), &1);
    }
}
