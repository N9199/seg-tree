use crate::nodes::Node;

/// Segment tree with range queries and point updates.
/// It uses `O(n)` space, assuming that each node uses `O(1)` space.
/// Note if you don't need to use `lower_bound`, just use the [`SegmentTree`](crate::segment_tree::SegmentTree) it uses half the memory and it's more performant.
pub struct Recursive<T: Node + Clone> {
    nodes: Vec<T>,
    n: usize,
}

impl<T> Recursive<T>
where
    T: Node + Clone,
{
    /// Builds segment tree from slice, each element of the slice will correspond to a leaf of the segment tree.
    /// It has time complexity of `O(n*log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        let mut nodes = Vec::with_capacity(4 * n);
        for _ in 0..4 {
            for v in values {
                nodes.push(v.clone());
            }
        }
        let mut out = Self { nodes, n };
        if n == 0 {
            return out;
        }
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

    /// Sets the p-th element of the segment tree to value T and update the segment tree correspondingly.
    /// It will panic if p is not in `[0,n)`
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    pub fn update(&mut self, p: usize, value: &<T as Node>::Value) {
        self.update_helper(p, value, 0, 0, self.n - 1);
    }

    fn update_helper(
        &mut self,
        p: usize,
        value: &<T as Node>::Value,
        curr_node: usize,
        i: usize,
        j: usize,
    ) {
        if j < p || p < i {
            return;
        }
        if i == j {
            self.nodes[curr_node] = Node::initialize(value);
            return;
        }
        let mid = (i + j) / 2;
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        self.update_helper(p, value, left_node, i, mid);
        self.update_helper(p, value, right_node, mid + 1, j);
        self.nodes[curr_node] = Node::combine(&self.nodes[left_node], &self.nodes[right_node]);
    }

    /// Returns the result from the range `[left,right]`.
    /// It returns None if and only if range is empty.
    /// It will **panic** if `left` or `right` are not in [0,n).
    /// It has time complexity of `O(log(n))`, assuming that [combine](Node::combine) has constant time complexity.
    #[allow(clippy::must_use_candidate)]
    pub fn query(&self, left: usize, right: usize) -> Option<T> {
        self.query_helper(left, right, 0, 0, self.n - 1)
    }

    fn query_helper(
        &self,
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
        if left <= i && j <= right {
            return Some(self.nodes[curr_node].clone());
        }
        match (
            self.query_helper(left, right, left_node, i, mid),
            self.query_helper(left, right, right_node, mid + 1, j),
        ) {
            (Some(ans_left), Some(ans_right)) => Some(T::combine(&ans_left, &ans_right)),
            (Some(ans_left), None) => Some(ans_left),
            (None, Some(ans_right)) => Some(ans_right),
            (None, None) => None,
        }
    }

    /// A method that finds the smallest prefix[^note] `u` such that `predicate(u.value(), value)` is `true`. The following must be true:
    /// - `predicate` is monotonic over prefixes[^note2].
    /// - `g` will satisfy the following, given segments `[i,j]` and `[i,k]` with `j<k` we have that `predicate([i,k].value(),value)` implies `predicate([j+1,k].value(),g([i,j].value(),value))`.
    ///
    /// These are two examples, the first is finding the smallest prefix which sums at least some value.
    /// ```
    /// # use seg_tree::{segment_tree::RecursiveSegmentTree,utils::Sum,nodes::Node};
    /// let predicate = |left_value:&usize, value:&usize|{*left_value>=*value}; // Is the sum greater or equal to value?
    /// let g = |left_node:&usize,value:usize|{value-*left_node}; // Subtract the sum of the prefix.
    /// # let nodes: Vec<Sum<usize>> = (0..10).map(|x| Sum::initialize(&x)).collect();
    /// let seg_tree = RecursiveSegmentTree::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with Sum<usize> nodes
    /// let index = seg_tree.lower_bound(predicate, g, 3); // Will return 2 as sum([0,1,2])>=3
    /// # let sums = vec![0,1,3,6,10,15,21,28,36,45];
    /// # for i in 0..10{
    /// #    assert_eq!(seg_tree.lower_bound(predicate, g, sums[i]), i);
    /// # }
    /// ```
    /// The second is finding the position of the smallest value greater or equal to some value.
    /// ```
    /// # use seg_tree::{segment_tree::RecursiveSegmentTree,utils::Max,nodes::Node};
    /// let predicate = |left_value:&usize, value:&usize|{*left_value>=*value}; // Is the maximum greater or equal to value?
    /// let g = |_left_node:&usize,value:usize|{value}; // Do nothing
    /// # let nodes: Vec<Max<usize>> = (0..10).map(|x| Max::initialize(&x)).collect();
    /// let seg_tree = RecursiveSegmentTree::build(&nodes); // [0,1,2,3,4,5,6,7,8,9] with Max<usize> nodes
    /// let index = seg_tree.lower_bound(predicate, g, 3); // Will return 3 as 3>=3
    /// # for i in 0..10{
    /// #    assert_eq!(seg_tree.lower_bound(predicate, g, i), i);
    /// # }
    /// ```
    ///
    /// [^note]: A prefix is a segment of the form `[0,i]`.
    ///
    /// [^note2]: Given two prefixes `u` and `v` if `u` is contained in `v` then `predicate(u.value(), value)` implies `predicate(v.value(), value)`.
    pub fn lower_bound<F, G>(&self, predicate: F, g: G, value: <T as Node>::Value) -> usize
    where
        F: Fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
        G: Fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    {
        self.lower_bound_helper(0, 0, self.n - 1, predicate, g, value)
    }
    fn lower_bound_helper<F, G>(
        &self,
        curr_node: usize,
        i: usize,
        j: usize,
        predicate: F,
        g: G,
        value: <T as Node>::Value,
    ) -> usize
    where
        F: Fn(&<T as Node>::Value, &<T as Node>::Value) -> bool,
        G: Fn(&<T as Node>::Value, <T as Node>::Value) -> <T as Node>::Value,
    {
        if i == j {
            return i;
        }
        let mid = (i + j) / 2;
        let left_node = 2 * curr_node + 1;
        let right_node = 2 * curr_node + 2;
        let left_value = self.nodes[left_node].value();
        if predicate(left_value, &value) {
            self.lower_bound_helper(left_node, i, mid, predicate, g, value)
        } else {
            let value = g(left_value, value);
            self.lower_bound_helper(right_node, mid + 1, j, predicate, g, value)
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::{nodes::Node, utils::Min};

    use super::Recursive;

    #[test]
    fn non_empty_query_returns_some() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Recursive::build(&nodes);
        assert!(segment_tree.query(0, 10).is_some());
    }
    #[test]
    fn empty_query_returns_none() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Recursive::build(&nodes);
        assert!(segment_tree.query(10, 0).is_none());
    }
    #[test]
    fn update_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let mut segment_tree = Recursive::build(&nodes);
        let value = 20;
        segment_tree.update(0, &value);
        assert_eq!(segment_tree.query(0, 0).unwrap().value(), &value);
    }
    #[test]
    fn query_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let segment_tree = Recursive::build(&nodes);
        assert_eq!(segment_tree.query(1, 10).unwrap().value(), &1);
    }
}
