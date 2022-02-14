use crate::nodes::{LazyNode, Node, PersistentNode};


/// Lazy persistent segment tree, it saves every version of itself, it has range queries and range updates.
/// It uses `O(n+q*log(n))` space, where `q` is the amount of updates, and assuming that each node uses `O(1)` space.
pub struct LazyPersistentSegmentTree<T: PersistentNode + LazyNode> {
    nodes: Vec<T>,
    roots: Vec<usize>,
    n: usize,
}

impl<T> LazyPersistentSegmentTree<T>
where
    T: PersistentNode + LazyNode + Clone, // + std::fmt::Debug,
{
    /// Builds a lazy persistent segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// It has time complexity of `O(n*log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut temp = Self {
            nodes: Vec::with_capacity(4*n),
            roots: Vec::with_capacity(1),
            n,
        };
        let root = temp.build_helper(values, 0, n - 1);
        temp.roots.push(root);
        temp
    }

    fn build_helper(&mut self, values: &[T], i: usize, j: usize) -> usize {
        let mid = (i + j) / 2;
        if i == j {
            let curr_node = self.nodes.len();
            self.nodes.push(values[i].clone());
            return curr_node;
        }
        let left_node = self.build_helper(values, i, mid);
        let right_node = self.build_helper(values, mid + 1, j);
        let curr_node = self.nodes.len();
        self.nodes
            .push(T::combine(&self.nodes[left_node], &self.nodes[right_node]));
        self.nodes[curr_node].set_children(left_node, right_node);
        curr_node
    }

    /// Returns the result from the range \[left,right\] from the version of the segment tree.
    /// It returns None if and only if range is empty.
    /// It will **panic** if left or right are not in [0,n), or if version is not in [0,[versions](LazyPersistentSegmentTree::versions)).
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine), [update_lazy_value](LazyNode::update_lazy_value) and [update_lazy_value](LazyNode::lazy_update) have constant time complexity.
    pub fn query(&mut self, version: usize, left: usize, right: usize) -> Option<T> {
        self.query_helper(self.roots[version], left, right, 0, self.n - 1)
    }

    fn push(&mut self, curr_node: usize, i: usize, j: usize) {
        if self.nodes[curr_node].lazy_value().is_some() && i != j {
            let left_node = self.nodes.len();
            let right_node = self.nodes.len() + 1;
            self.nodes
                .push(self.nodes[self.nodes[curr_node].left_child()].clone());
            self.nodes
                .push(self.nodes[self.nodes[curr_node].right_child()].clone());
            let (parent_slice, sons_slice) = self.nodes.split_at_mut(curr_node + 1);
            let value = parent_slice[curr_node].lazy_value().unwrap();
            sons_slice[left_node - curr_node - 1].update_lazy_value(value);
            sons_slice[right_node - curr_node - 1].update_lazy_value(value);
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
    ) -> Option<T> {
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
        let left_node = self.nodes[curr_node].left_child();
        let right_node = self.nodes[curr_node].right_child();
        match (
            self.query_helper(left_node, left, right, i, mid),
            self.query_helper(right_node, left, right, mid + 1, j),
        ) {
            (Some(ans_left), Some(ans_right)) => Some(T::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }

    // Creates a new segment tree version from version were the p-th element of the segment tree to value T and update the segment tree correspondingly.
    /// It will panic if p is not in \[0,n), or if version is not in [0,[versions](LazyPersistentSegmentTree::versions)).
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine), [update_lazy_value](LazyNode::update_lazy_value) and [update_lazy_value](LazyNode::lazy_update) have constant time complexity.
    pub fn update(&mut self, version: usize, left: usize, right: usize, value: <T as Node>::Value) {
        let new_root = self.update_helper(self.roots[version], left, right, &value, 0, self.n - 1);
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
        let left_node = self.update_helper(self.nodes[x].left_child(), left, right, value, i, mid);
        let right_node =
            self.update_helper(self.nodes[x].right_child(), left, right, value, mid + 1, j);
        self.nodes[x] = Node::combine(&self.nodes[left_node], &self.nodes[right_node]);
        self.nodes[x].set_children(left_node, right_node);
        x
    }

    /// Return the amount of different versions the current segment tree has.
    pub fn versions(&self) -> usize {
        self.roots.len()
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        default::Sum, nodes::Node, segment_tree::persistent_lazy::LazyPersistentSegmentTree,
    };

    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 10, 0).is_none());
    }
    #[test]
    fn normal_update_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, 0, value);
        assert_eq!(segment_tree.query(1, 0, 0).unwrap().value(), &value);
    }

    #[test]
    fn branched_update_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, 10, value);
        segment_tree.update(0, 1, 1, value);
        assert_eq!(segment_tree.query(2, 0, 0).unwrap().value(), &0);
        assert_eq!(segment_tree.query(2, 1, 1).unwrap().value(), &(value + 1));
    }

    #[test]
    fn query_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        assert_eq!(segment_tree.query(0, 0, 10).unwrap().value(), &55);
    }
}
