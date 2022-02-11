use crate::nodes::{LazyNode, Node, PersistentNode};

pub struct LazyPersistentSegmentTree<T: PersistentNode> {
    nodes: Vec<T>,
    roots: Vec<usize>,
    n: usize,
}

impl<T> LazyPersistentSegmentTree<T>
where
    T: PersistentNode + LazyNode + Clone, // + std::fmt::Debug,
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
            // log::debug!(
            // "[BUILD] Node {curr_node} [{i},{j}]: {:?}",
            // &self.nodes[curr_node]
            // );
            return curr_node;
        }
        let left_node = self.build_helper(values, i, mid);
        let right_node = self.build_helper(values, mid + 1, j);
        let curr_node = self.nodes.len();
        self.nodes
            .push(T::combine(&self.nodes[left_node], &self.nodes[right_node]));
        self.nodes[curr_node].set_sons(left_node, right_node);
        // log::debug!(
        // "[BUILD] Node {curr_node} [{i},{j}]: {:?}",
        // &self.nodes[curr_node]
        // );
        curr_node
    }

    pub fn query(&mut self, version: usize, left: usize, right: usize) -> Option<T> {
        self.query_helper(self.roots[version], left, right, 0, self.n - 1)
    }

    fn push(&mut self, curr_node: usize, i: usize, j: usize) {
        if self.nodes[curr_node].lazy_value().is_some() && i != j {
            let left_node = self.nodes.len();
            let right_node = self.nodes.len() + 1;
            self.nodes
                .push(self.nodes[self.nodes[curr_node].left()].clone());
            self.nodes
                .push(self.nodes[self.nodes[curr_node].right()].clone());
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
        // log::debug!(
        // "[QUERY] Node {curr_node} [{i},{j}]: {:?}",
        // &self.nodes[curr_node]
        // );
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
            // log::debug!(
            // "[UPDATE] (Interior) Node {x} [{i},{j}]: {:?}",
            // &self.nodes[x]
            // );
            return x;
        }
        let mid = (i + j) / 2;
        let left_node = self.update_helper(self.nodes[x].left(), left, right, value, i, mid);
        let right_node = self.update_helper(self.nodes[x].right(), left, right, value, mid + 1, j);
        self.nodes[x] = Node::combine(&self.nodes[left_node], &self.nodes[right_node]);
        self.nodes[x].set_sons(left_node, right_node);
        // log::debug!("[UPDATE] (Normal) Node {x} [{i},{j}]: {:?}", &self.nodes[x]);
        x
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        default::Sum, nodes::Node, segment_tree::persistent_lazy::LazyPersistentSegmentTree,
    };

    fn init() {
        let _ =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .is_test(true)
                .format_timestamp(None)
                .try_init();
    }

    #[test]
    fn non_empty_query_returns_some() {
        init();
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        init();
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 10, 0).is_none());
    }
    #[test]
    fn normal_update_works() {
        init();
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        let value = 20;
        log::debug!("[TEST] Update: version=0 [0,0] value={value}");
        segment_tree.update(0, 0, 0, value);
        log::debug!("[TEST] Query: version=1 [0,0]");
        assert_eq!(segment_tree.query(1, 0, 0).unwrap().value(), &value);
    }

    #[test]
    fn branched_update_works() {
        init();
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        let value = 20;
        log::debug!("[TEST] Update: version=0 [0,10] value={value}");
        segment_tree.update(0, 0, 10, value);
        log::debug!("[TEST] Update: version=0 [1,1] value={value}");
        segment_tree.update(0, 1, 1, value);
        log::debug!("[TEST] Query: version=2 [0,0]");
        assert_eq!(segment_tree.query(2, 0, 0).unwrap().value(), &0);
        log::debug!("[TEST] Query: version=2 [1,1]");
        assert_eq!(segment_tree.query(2, 1, 1).unwrap().value(), &(value + 1));
    }

    #[test]
    fn query_works() {
        init();
        let nodes: Vec<Sum<usize>> = (0..=10).map(|x| Sum::initialize(&x)).collect();
        let mut segment_tree = LazyPersistentSegmentTree::build(&nodes);
        assert_eq!(segment_tree.query(0, 0, 10).unwrap().value(), &55);
    }
}
