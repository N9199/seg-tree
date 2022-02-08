use crate::nodes::Node;

pub struct IterativeSegmentTree<T: Node> {
    nodes: Vec<T>,
    n: usize,
}

impl<T: Node + Clone> IterativeSegmentTree<T> {
    /// Builds Iterative Segment Tree from slice.
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut nodes = Vec::with_capacity(2 * n);
        for _ in 0..2 {
            for v in values{
                nodes.push(v.clone());
            }
        }
        (1..n).rev().for_each(|i| {
            nodes[i] = T::combine(&nodes[2 * i], &nodes[2 * i + 1]);
        });
        Self { nodes, n }
    }

    /// Sets the i-th element of the segment tree to value T and update the segment tree correspondingly.
    /// It will panic if i is not in [0,n)
    pub fn set(&mut self, i: usize, value: T) {
        let mut i = i;
        i += self.n;
        self.nodes[i] = value;
        while i > 0 {
            i >>= 1;
            self.nodes[i] = T::combine(&self.nodes[2 * i], &self.nodes[2 * i + 1]);
        }
    }

    /// Returns the result from the range [l,r].
    /// It returns None if and only if range is empty.
    /// It will **panic** if l or r are not in [0,n).
    pub fn query(&self, l: usize, r: usize) -> Option<T> {
        let (mut l, mut r) = (l, r);
        let mut ansl: Option<T> = None;
        let mut ansr: Option<T> = None;
        l += self.n;
        r += self.n+1;
        while l < r {
            if l & 1 != 0 {
                ansl = Some(match ansl {
                    None => T::initialize(self.nodes[l].values()),
                    Some(node) => T::combine(&node, &self.nodes[l]),
                });
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                ansr = Some(match ansr {
                    None => T::initialize(self.nodes[r].values()),
                    Some(node) => T::combine(&self.nodes[r], &node),
                });
            }
            l >>= 1;
            r >>= 1;
        }
        match (ansl, ansr) {
            (Some(ansl), Some(ansr)) => Some(T::combine(&ansl, &ansr)),
            (Some(ansl), None) => Some(T::initialize(ansl.values())),
            (None, Some(ansr)) => Some(T::initialize(ansr.values())),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{default::Min, nodes::Node};

    use super::IterativeSegmentTree;

    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x|Min::initialize(&x)).collect();
        let segment_tree = IterativeSegmentTree::build(&nodes);
        assert!(segment_tree.query(0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x|Min::initialize(&x)).collect();
        let segment_tree = IterativeSegmentTree::build(&nodes);
        assert!(segment_tree.query(10, 0).is_none());
    }
    #[test]
    fn set_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x|Min::initialize(&x)).collect();
        let mut segment_tree = IterativeSegmentTree::build(&nodes);
        let value = 20;
        segment_tree.set(0, Min::initialize(&value));
        assert_eq!(segment_tree.query(0, 0).unwrap().values(), &value);
    }
    #[test]
    fn query_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x|Min::initialize(&x)).collect();
        let segment_tree = IterativeSegmentTree::build(&nodes);
        assert_eq!(segment_tree.query(1, 10).unwrap().values(), &1);
    }
}
