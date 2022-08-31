use std::marker::PhantomData;

use bit_vec::BitVec;

use super::persistent_utils::PersistentWrapper;

pub struct NodeKey {
    pub i: usize,
    pub j: usize,
}

impl core::fmt::Debug for NodeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}, {}]", self.i, self.j))
    }
}

pub struct DbgTree<'a, A, B, F>
where
    A: core::fmt::Debug + 'a,
    F: Fn(&'a [B], &mut dyn FnMut(usize, usize, &'a A)),
{
    nodes: &'a [B],
    visitor: F,
    _phantom_data: PhantomData<A>,
}
#[inline]
pub const fn as_dbg_tree<'a, A, B, F>(nodes: &'a [B], visitor: F) -> DbgTree<'a, A, B, F>
where
    A: core::fmt::Debug + 'a,
    F: Fn(&'a [B], &mut dyn FnMut(usize, usize, &'a A)),
{
    DbgTree {
        nodes,
        visitor,
        _phantom_data: PhantomData,
    }
}

impl<'a, A, B, F> core::fmt::Debug for DbgTree<'a, A, B, F>
where
    A: core::fmt::Debug + 'a,
    F: Fn(&'a [B], &mut dyn FnMut(usize, usize, &'a A)),
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatter = f.debug_map();
        let mut f = |i, j, value: &A| {
            formatter.entry(&NodeKey { i, j }, value);
        };
        (self.visitor)(self.nodes, &mut f);
        formatter.finish()
    }
}
#[inline]
pub fn recursive_visitor<'a, T>(
    curr_node: usize,
    i: usize,
    j: usize,
    f: &mut dyn FnMut(usize, usize, &'a T),
    nodes: &'a [T],
) where
    T: core::fmt::Debug,
{
    f(i, j, &nodes[curr_node]);
    if i == j {
        return;
    }
    let mid = (i + j) / 2;
    recursive_visitor(2 * curr_node + 1, i, mid, f, nodes);
    recursive_visitor(2 * curr_node + 2, mid + 1, j, f, nodes);
}

pub fn persistent_visitor<'a, 'b, T>(
    curr_node: usize,
    i: usize,
    j: usize,
    f: &mut dyn FnMut(usize, usize, &'a T),
    nodes: &'a [PersistentWrapper<T>],
    visited: &'b mut BitVec,
) where
    T: core::fmt::Debug,
{
    f(i, j, nodes[curr_node].get_inner());
    visited.set(curr_node, true);
    if i == j {
        return;
    }
    let mid = (i + j) / 2;
    let left_node = nodes[curr_node].left_child().unwrap().get();
    let right_node = nodes[curr_node].right_child().unwrap().get();
    if !visited[left_node] {
        persistent_visitor(left_node, i, mid, f, nodes, visited);
    }
    if !visited[right_node] {
        persistent_visitor(right_node, mid + 1, j, f, nodes, visited);
    }
}

pub fn lazy_persistent_visitor<'a, 'b, T>(
    curr_node: usize,
    i: usize,
    j: usize,
    f: &mut dyn FnMut(usize, usize, &'a T),
    nodes: &'a [PersistentWrapper<T>],
    visited: &'b mut BitVec,
) where
    T: core::fmt::Debug,
{
    f(i, j, nodes[curr_node].get_inner());
    visited.set(curr_node, true);
    if i == j {
        return;
    }
    let mid = (i + j) / 2;
    if let Some(left_node) = nodes[curr_node].left_child() {
        let left_node = left_node.get();
        if !visited[left_node] {
            lazy_persistent_visitor(left_node, i, mid, f, nodes, visited);
        }
    }
    if let Some(right_node) = nodes[curr_node].right_child() {
        let right_node = right_node.get();
        if !visited[right_node] {
            lazy_persistent_visitor(right_node, mid + 1, j, f, nodes, visited);
        }
    }
}
