use crate::nodes::PersistentNode;

pub struct PersistentSegmentTree<T: PersistentNode> {
    nodes: Vec<T>,
    roots: Vec<usize>,
    n: usize,
}

impl<T: PersistentNode + Clone> PersistentSegmentTree<T> {
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut temp = Self {
            nodes: Vec::new(),
            roots: Vec::new(),
            n,
        };
        temp.build_helper(values, 0, n - 1);
        temp
    }

    fn build_helper(&mut self, values: &[T], i: usize, j: usize) -> usize {
        let curr_node = self.nodes.len();
        let mid = (i + j) / 2;
        if i == j {
            self.nodes.push(values[i].clone());
            return curr_node;
        }
        let l = self.build_helper(values, i, mid);
        let r = self.build_helper(values, mid + 1, j);
        self.nodes.push(T::combine(&self.nodes[l], &self.nodes[r]));
        let last = self.nodes.len() - 1;
        self.nodes[last].set_sons(l, r);
        curr_node
    }

    pub fn query(&self, t: usize, l: usize, r: usize) -> Option<T> {
        self.query_helper(self.roots[t], l, r, 0, self.n - 1)
    }

    fn query_helper(&self, curr_node: usize, l: usize, r: usize, i: usize, j: usize) -> Option<T> {
        if j < l || r < i {
            return None;
        }
        if l <= i && j <= r {
            return Some(self.nodes[curr_node].clone());
        }
        let mid = (i + j) / 2;
        let left_node = self.nodes[curr_node].left();
        let right_node = self.nodes[curr_node].right();
        let ans_left = self.query_helper(left_node, l, r, i, mid);
        let ans_right = self.query_helper(right_node, l, r, mid + 1, j);
        match (ans_left, ans_right) {
            (Some(ans_left), Some(ans_right)) => Some(T::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }
}
