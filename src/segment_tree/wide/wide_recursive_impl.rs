use core::mem::MaybeUninit;

use crate::{internal_utils::dbg_utils::as_dbg_tree, nodes::WideNode};

pub struct WideRecursive<const B: usize, T> {
    nodes: Vec<T>,
    n: usize,
}

impl<const B: usize, T> WideRecursive<B, T>
where
    T: WideNode<B> + Clone,
{
    #[must_use]
    pub fn build(values: &[T]) -> Self {
        let n = values.len();
        if n == 0 {
            return Self {
                nodes: Vec::new(),
                n,
            };
        }
        // sum j=1 to height of B^{j-1} = (1-B^height)/(1-B) <= 4*n
        let vec_len = 8 * n;
        let mut nodes = Vec::with_capacity(vec_len);
        // SAFETY: As the nodes are of type `MaybeUninit<T>` this is safe.
        unsafe { nodes.set_len(vec_len) };
        Self::build_helper(0, 0, n - 1, values, &mut nodes);
        let ptr = nodes.as_mut_ptr();
        let len = nodes.len();
        let cap = nodes.capacity();
        debug_assert_eq!(len, cap);
        debug_assert_eq!(len, 4 * n);
        core::mem::forget(nodes);
        let nodes = unsafe { Vec::from_raw_parts(ptr.cast::<T>(), len, cap) };

        Self { nodes, n }
    }

    #[inline]
    fn build_helper(
        curr_node: usize,
        i: usize,
        j: usize,
        values: &[T],
        nodes: &mut [MaybeUninit<T>],
    ) {
        if j - i < B - 1 {
            for k in 0..=((B - 1).min(j - i)) {
                debug_assert!(
                    curr_node + k < nodes.len(),
                    "curr_node: {curr_node}, k: {k} len: {}",
                    nodes.iter().len()
                );
                debug_assert!(i + k < values.len(), "i: {i}, k: {k}");
                nodes[curr_node + k].write(values[i + k].clone());
            }
        }
        let dx = (j - i) / B;
        let mut new_node = B * curr_node + 1;
        let mut left = i;
        let mut right = i + dx;
        Self::build_helper(new_node, left, right, values, nodes);
        for k in 2..B {
            new_node += 1;
            left = right + 1;
            right = i + k * dx;
            Self::build_helper(new_node, left, right, values, nodes);
        }
        Self::build_helper(new_node + 1, right + 1, j, values, nodes);
        let (top_nodes, bottom_nodes) = nodes.split_at_mut(curr_node + 1);
        top_nodes[curr_node].write(WideNode::combine_multiple(
            bottom_nodes[((B - 1) * curr_node)..((B - 1) * curr_node + B)]
                .as_array::<B>()
                .map(|array| unsafe { core::mem::transmute(array) })
                .unwrap(),
        ));
    }
}

impl<const B: usize, T> core::fmt::Debug for WideRecursive<B, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WideRecursive")
            .field("n", &self.n)
            .field(
                "nodes",
                &as_dbg_tree(&self.nodes, |nodes, f| {
                    wide_recursive_visitor::<B, T>(0, 0, self.n - 1, f, nodes)
                }),
            )
            .finish()
    }
}

#[inline]
pub fn wide_recursive_visitor<'a, const B: usize, T>(
    curr_node: usize,
    i: usize,
    j: usize,
    f: &mut dyn FnMut(usize, usize, &'a T),
    nodes: &'a [T],
) where
    T: core::fmt::Debug,
{
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{
        nodes::{Node as _, WideNode},
        utils::Min,
    };

    use super::WideRecursive;

    // Smoke test: building from a non-empty slice should not panic.
    #[test]
    fn build_non_empty_does_not_panic() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let _tree = WideRecursive::<2, _>::build(&nodes);
    }

    // Building from an empty slice is a valid, defined edge case.
    #[test]
    fn build_empty_does_not_panic() {
        let nodes: Vec<Min<usize>> = vec![];
        let _tree = WideRecursive::<2, _>::build(&nodes);
    }

    // A single-element slice is the minimal non-empty case: just a root leaf.
    #[test]
    fn build_single_element_does_not_panic() {
        let nodes: Vec<Min<usize>> = vec![Min::initialize(&42)];
        let _tree = WideRecursive::<2, _>::build(&nodes);
    }

    // Verify B=2 degenerates to binary behaviour: `n` is stored correctly.
    #[test]
    fn build_stores_correct_n_binary() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let tree = WideRecursive::<2, _>::build(&nodes);
        assert_eq!(tree.n, 11);
    }

    // Same sanity check for a wider branching factor.
    #[test]
    fn build_stores_correct_n_ternary() {
        let nodes: Vec<Min<usize>> = (0..=11).map(|x| Min::initialize(&x)).collect();
        let tree = WideRecursive::<3, _>::build(&nodes);
        assert_eq!(tree.n, 12);
    }

    // A power-of-B sized input is the "perfect tree" case and exercises
    // the balanced split path in build_helper.
    #[test]
    fn build_power_of_b_size() {
        // 3^3 = 27 leaves → perfectly balanced ternary tree
        let nodes: Vec<Min<usize>> = (0..27).map(|x| Min::initialize(&x)).collect();
        let _tree = WideRecursive::<3, _>::build(&nodes);
    }

    // n = B-1 is the smallest input that hits the leaf short-circuit in
    // build_helper (`j - i < B - 1` is true for the whole slice).
    #[test]
    fn build_exactly_b_minus_one_elements() {
        // B=4, so 3 elements → entire slice is one "wide leaf"
        let nodes: Vec<Min<usize>> = (0..3).map(|x| Min::initialize(&x)).collect();
        let _tree = WideRecursive::<4, _>::build(&nodes);
    }

    // n = B is the first case that exercises an internal node *and* leaf
    // children (the recursion splits into B sub-ranges of size 1).
    #[test]
    fn build_exactly_b_elements() {
        let nodes: Vec<Min<usize>> = (0..4).map(|x| Min::initialize(&x)).collect();
        let _tree = WideRecursive::<4, _>::build(&nodes);
    }

    // Large B: confirm the tree still builds without index panics even
    // when B is close to n.
    #[test]
    fn build_large_branching_factor() {
        let nodes: Vec<Min<usize>> = (0..16).map(|x| Min::initialize(&x)).collect();
        let _tree = WideRecursive::<8, _>::build(&nodes);
    }

    // Once wide_recursive_visitor is implemented this test will assert the
    // Debug output matches the expected pre-order traversal string, mirroring
    // the `dbg_works` test in recursive.rs.  For now it just checks that
    // Debug doesn't panic (the `todo!()` inside the visitor will panic, so
    // this test is marked `#[ignore]` until the visitor is filled in).
    #[test]
    #[ignore = "wide_recursive_visitor is not yet implemented (todo!)"]
    fn dbg_works() {
        let nodes: Vec<Min<usize>> = (0..=10).map(|x| Min::initialize(&x)).collect();
        let tree = WideRecursive::<2, _>::build(&nodes);
        let _dbg = format!("{tree:?}");
        // Once the visitor is done, replace the line above with something like:
        // let expected = "WideRecursive { n: 11, nodes: { ... } }";
        // assert_eq!(_dbg, expected);
    }
}
