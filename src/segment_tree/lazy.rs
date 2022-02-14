use crate::nodes::{LazyNode, Node};

/// Implementation of lazy segment tree with range queries and range updates.
pub struct LazySegmentTree<T: LazyNode> {
    nodes: Vec<T>,
    n: usize,
}

impl<T: LazyNode + Clone> LazySegmentTree<T> {
    /// Builds lazy segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
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

    /// Updates the range \[i,j\] with value.
    /// It will panic if i or j is not in \[0,n)
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
    /// It will **panic** if left or right are not in [0,n).
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
            self.query_helper(left, right, right_node, mid + 1, right),
        ) {
            (Some(ans_left), Some(ans_right)) => Some(T::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{default::Min, nodes::Node};

    use super::LazySegmentTree;
    // TODO Add more tests
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
