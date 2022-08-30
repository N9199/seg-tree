use core::mem::MaybeUninit;

use crate::{internal_utils::as_dbg_tree, nodes::Node};

/// Segment tree with range queries and point updates.
/// It uses `O(n)` space, assuming that each node uses `O(1)` space.
/// Note if you need to use `lower_bound`, just use the [`RecursiveSegmentTree`](crate::segment_tree::RecursiveSegmentTree) it uses double the memory though and it's less performant.
pub struct Iterative<T> {
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
        let mut nodes: Vec<MaybeUninit<T>> = Vec::with_capacity(2 * n);
        unsafe { nodes.set_len(2 * n) };
        for i in 0..n {
            nodes[i + n].write(values[i].clone());
        }
        for i in (1..n).rev() {
            let (bottom_nodes, top_nodes) = nodes.split_at_mut(i + 1);
            bottom_nodes[i].write(Node::combine(
                unsafe { top_nodes[i - 1].assume_init_ref() },
                unsafe { top_nodes[i].assume_init_ref() },
            ));
        }
        let ptr = nodes.as_mut_ptr();
        core::mem::forget(nodes);
        let nodes = unsafe { Vec::from_raw_parts(ptr.cast(), 2 * n, 2 * n) };
        Self { nodes, n }
    }

    /// Sets the i-th element of the segment tree to value T and update the segment tree correspondingly.
    /// It will panic if i is not in `[0,n)`
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    pub fn update(&mut self, i: usize, value: &<T as Node>::Value) {
        let mut i = i;
        i += self.n;
        self.nodes[i] = Node::initialize(value);
        i >>= 1;
        while i > 0 {
            self.nodes[i] = Node::combine(&self.nodes[2 * i], &self.nodes[2 * i + 1]);
            i >>= 1;
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

impl<T> Iterative<T>
where
    T: Node + core::fmt::Debug,
{
    fn dbg_visitor<'a>(n: usize, f: &mut dyn FnMut(usize, usize, &'a T), nodes: &'a [T]) {
        let mut segments = vec![(0, 0); 2 * n];
        for i in 0..n {
            segments[i + n] = (i, i);
            f(i, i, &nodes[n + i]);
        }
        let helper = |(a1, b1): (usize, usize), (a2, b2)| (a1.min(a2), b1.max(b2));
        for i in (1..n).rev() {
            segments[i] = helper(segments[2 * i], segments[2 * i + 1]);
            f(segments[i].0, segments[i].1, &nodes[i]);
        }
    }
}

impl<T> core::fmt::Debug for Iterative<T>
where
    T: Node + core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Recursive")
            .field("n", &self.n)
            .field(
                "nodes",
                &as_dbg_tree(&self.nodes, |nodes, f| {
                    Self::dbg_visitor(self.n, f, nodes);
                }),
            )
            .finish()
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
        for i in 0..10 {
            assert_eq!(segment_tree.query(i, 10).unwrap().value(), &i);
        }
    }
}
