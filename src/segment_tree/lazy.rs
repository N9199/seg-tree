use crate::nodes::{LazyNode, Node};

pub struct LazySegmentTree<T: LazyNode> {
    nodes: Vec<T>,
    n: usize,
}

impl<T: LazyNode + Clone> LazySegmentTree<T> {
    /// Builds Lazy Segment Tree from slice.
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

    fn build_helper(&mut self, u: usize, i: usize, j: usize, values: &[T]) {
        if i == j {
            self.nodes[u] = values[i].clone();
            return;
        }
        let m = (i + j) / 2;
        let l = 2 * u + 1;
        let r = 2 * u + 2;
        self.build_helper(l, i, m, values);
        self.build_helper(r, m + 1, j, values);
        self.nodes[u] = T::combine(&self.nodes[l], &self.nodes[r]);
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

    /// Updates the range [i,j] with value.
    /// It will panic if i or j is not in [0,n)
    pub fn update(&mut self, i: usize, j: usize, value: <T as Node>::Value) {
        self.update_helper(i, j, &value, 0, 0, self.n - 1);
    }

    fn update_helper(
        &mut self,
        l: usize,
        r: usize,
        value: &<T as Node>::Value,
        u: usize,
        i: usize,
        j: usize,
    ) {
        if self.nodes[u].lazy_value().is_some() {
            self.push(u, i, j);
        }
        if j < l || r < i {
            return;
        }
        if l <= i && j <= r {
            self.nodes[u].update_lazy_value(value);
            self.push(u, i, j);
            return;
        }
        let m = (i + j) / 2;
        let left = 2 * u + 1;
        let right = 2 * u + 2;
        self.update_helper(l, r, value, left, i, m);
        self.update_helper(l, r, value, right, m + 1, j);
        self.nodes[u] = T::combine(&self.nodes[l], &self.nodes[r]);
    }

    /// Returns the result from the range \[l,r\].
    /// It returns None if and only if range is empty.
    /// It will **panic** if l or r are not in [0,n).
    pub fn query(&mut self, l: usize, r: usize) -> Option<T> {
        self.query_helper(l, r, 0, 0, self.n - 1)
    }

    fn query_helper(&mut self, l: usize, r: usize, u: usize, i: usize, j: usize) -> Option<T> {
        if j < l || r < i {
            return None;
        }
        let m = (i + j) / 2;
        let left = 2 * u + 1;
        let right = 2 * u + 2;
        if self.nodes[u].lazy_value().is_some() {
            self.push(u, i, j);
        }
        if l <= i && j <= r {
            return Some(self.nodes[u].clone());
        }
        let ansl = self.query_helper(l, r, left, i, m);
        let ansr = self.query_helper(l, r, right, m + 1, r);
        match (ansl, ansr) {
            (Some(ansl), Some(ansr)) => Some(T::combine(&ansl, &ansr)),
            (Some(ansl), None) => Some(ansl),
            (None, Some(ansr)) => Some(ansr),
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
        assert_eq!(segment_tree.query(0, 1).unwrap().values(), &value);
    }
    #[test]
    fn query_works() {
        let nodes: Vec<Min<usize>> = (0..10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = LazySegmentTree::build(&nodes);
        assert_eq!(segment_tree.query(1, 9).unwrap().values(), &1);
    }
}
