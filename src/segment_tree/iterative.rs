use crate::nodes::Node;

/// Segment tree with range queries and point updates.
/// It uses `O(n)` space, assuming that each node uses `O(1)` space.
/// Note if you need to use `lower_bound`, just use the [`RecursiveSegmentTree`](crate::segment_tree::RecursiveSegmentTree) it uses double the memory though and it's less performant.
pub struct Iterative<T: Node + Clone> {
    nodes: Vec<T>,
    n: usize,
}

impl<T> Iterative<T>
where
    T: Node + Clone,
{
    /// Builds segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// It has time complexity of `O(n*log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut nodes = Vec::with_capacity(2 * n);
        for _ in 0..2 {
            for v in values {
                nodes.push(v.clone());
            }
        }
        (1..n).rev().for_each(|i| {
            nodes[i] = Node::combine(&nodes[2 * i], &nodes[2 * i + 1]);
        });
        Self { nodes, n }
    }

    /// Sets the i-th element of the segment tree to value T and update the segment tree correspondingly.
    /// It will panic if i is not in `[0,n)`
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    pub fn update(&mut self, i: usize, value: &<T as Node>::Value) {
        let mut i = i;
        i += self.n;
        self.nodes[i] = Node::initialize(value);
        while i > 0 {
            i >>= 1;
            self.nodes[i] = Node::combine(&self.nodes[2 * i], &self.nodes[2 * i + 1]);
        }
    }

    /// Returns the result from the range `[left,right]`.
    /// It returns None if and only if range is empty.
    /// It will **panic** if left or right are not in `[0,n)`.
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    #[allow(clippy::must_use_candidate)]
    pub fn query(&self, l: usize, r: usize) -> Option<T> {
        let (mut l, mut r) = (l, r);
        let mut ans_left = None;
        let mut ans_right = None;
        l += self.n;
        r += self.n + 1;
        while l < r {
            if l & 1 != 0 {
                ans_left = Some(match ans_left {
                    None => Node::initialize(self.nodes[l].value()),
                    Some(node) => Node::combine(&node, &self.nodes[l]),
                });
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                ans_right = Some(match ans_right {
                    None => Node::initialize(self.nodes[r].value()),
                    Some(node) => Node::combine(&self.nodes[r], &node),
                });
            }
            l >>= 1;
            r >>= 1;
        }
        match (ans_left, ans_right) {
            (Some(ans_left), Some(ans_right)) => Some(Node::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(Node::initialize(ans_left.value())),
            (None, Some(ans_right)) => Some(Node::initialize(ans_right.value())),
            (None, None) => None,
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{nodes::Node, utils::Min};

    use super::Iterative;

    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Iterative::build(&nodes);
        assert!(segment_tree.query(0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Iterative::build(&nodes);
        assert!(segment_tree.query(10, 0).is_none());
    }
    #[test]
    fn update_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = Iterative::build(&nodes);
        let value = 20;
        segment_tree.update(0, &value);
        assert_eq!(segment_tree.query(0, 0).unwrap().value(), &value);
    }
    #[test]
    fn query_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Iterative::build(&nodes);
        assert_eq!(segment_tree.query(1, 10).unwrap().value(), &1);
    }
}
