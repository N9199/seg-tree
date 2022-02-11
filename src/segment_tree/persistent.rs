use crate::nodes::{Node, PersistentNode};

pub struct PersistentSegmentTree<T: PersistentNode> {
    nodes: Vec<T>,
    roots: Vec<usize>,
    n: usize,
}

impl<T> PersistentSegmentTree<T>
where
    T: PersistentNode + Clone,
{
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut temp = Self {
            nodes: Vec::new(),
            roots: Vec::new(),
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
        self.nodes[curr_node].set_sons(left_node, right_node);
        curr_node
    }

    pub fn query(&self, version: usize, left: usize, right: usize) -> Option<T> {
        self.query_helper(self.roots[version], left, right, 0, self.n - 1)
    }

    fn query_helper(
        &self,
        curr_node: usize,
        left: usize,
        right: usize,
        i: usize,
        j: usize,
    ) -> Option<T> {
        if j < left || right < i {
            return None;
        }
        if left <= i && j <= right {
            return Some(self.nodes[curr_node].clone());
        }
        let mid = (i + j) / 2;
        let left_node = self.nodes[curr_node].left();
        let right_node = self.nodes[curr_node].right();
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

    pub fn update(&mut self, version: usize, p: usize, value: <T as Node>::Value) {
        let new_root = self.update_helper(self.roots[version], p, &value, 0, self.n - 1);
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
        let mid = (i + j) / 2;
        let left_node = self.update_helper(self.nodes[x].left(), p, value, i, mid);
        let right_node = self.update_helper(self.nodes[x].right(), p, value, mid + 1, j);
        self.nodes[x] = Node::combine(&self.nodes[left_node], &self.nodes[right_node]);
        self.nodes[x].set_sons(left_node, right_node);
        x
    }
}
#[cfg(test)]
mod tests {
    use crate::{default::Sum, nodes::Node, segment_tree::PersistentSegmentTree};

    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let segment_tree = PersistentSegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let segment_tree = PersistentSegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 10, 0).is_none());
    }
    #[test]
    fn normal_update_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = PersistentSegmentTree::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, value);
        assert_eq!(segment_tree.query(1, 0, 0).unwrap().value(), &value);
    }

    #[test]
    fn branched_update_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = PersistentSegmentTree::build(&nodes);
        let value = 20;
        segment_tree.update(0, 0, value);
        segment_tree.update(0, 1, value);
        assert_eq!(segment_tree.query(2, 0, 0).unwrap().value(), &0);
        assert_eq!(segment_tree.query(2, 1, 1).unwrap().value(), &value);
    }

    #[test]
    fn query_works() {
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let segment_tree = PersistentSegmentTree::build(&nodes);
        assert_eq!(segment_tree.query(0, 0, 10).unwrap().value(), &55);
    }
}
