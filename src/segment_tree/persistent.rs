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
        let u = self.nodes.len();
        let m = (i + j) / 2;
        if i == j {
            self.nodes.push(values[i].clone());
            return u;
        }
        let l = self.build_helper(values, i, m);
        let r = self.build_helper(values, m + 1, j);
        self.nodes.push(T::combine(&self.nodes[l], &self.nodes[r]));
        let last = self.nodes.len() - 1;
        self.nodes[last].set_sons(l, r);
        u
    }

    pub fn query(&self, t: usize, l: usize, r: usize) -> Option<T> {
        self.query_helper(self.roots[t], l, l, 0, self.n - 1)
    }

    fn query_helper(&self, u: usize, l: usize, r: usize, i: usize, j: usize) -> Option<T> {
        if j < l || r < i {
            return None;
        }
        if l <= i && j <= r {
            return Some(self.nodes[u].clone());
        }
        let m = (i + j) / 2;
        let left = self.nodes[u].left();
        let right = self.nodes[u].right();
        let ansl = self.query_helper(left, l, r, i, m);
        let ansr = self.query_helper(right, l, r, m + 1, j);
        match (ansl, ansr) {
            (Some(ansl), Some(ansr)) => Some(T::combine(&ansl, &ansr)),
            (Some(ansl), None) => Some(ansl),
            (None, Some(ansr)) => Some(ansr),
            (None, None) => None,
        }
    }
}
