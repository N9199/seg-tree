use crate::nodes::{LazyNode, Node, PersistentNode};

/// A simple wrapper for nodes to easily implement [PersistentNode]. If the wrapped node implements [LazyNode] the wrapper also implements it.
#[derive(Clone)]
pub struct PersistentWrapper<T>
where
    T: Node,
{
    node: T,
    left: usize,
    right: usize,
}

impl<T> Node for PersistentWrapper<T>
where
    T: Node,
{
    type Value = T::Value;

    fn initialize(value: &Self::Value) -> Self {
        Self {
            node: T::initialize(value),
            left: 0,
            right: 0,
        }
    }

    fn combine(a: &Self, b: &Self) -> Self {
        Self {
            node: T::combine(&a.node, &b.node),
            left: 0,
            right: 0,
        }
    }

    fn value(&self) -> &Self::Value {
        self.node.value()
    }
}
impl<T> LazyNode for PersistentWrapper<T>
where
    T: LazyNode,
{
    fn lazy_update(&mut self, i: usize, j: usize) {
        self.node.lazy_update(i, j);
    }

    fn update_lazy_value(&mut self, new_value: &<Self as Node>::Value) {
        self.node.update_lazy_value(new_value);
    }

    fn lazy_value(&self) -> Option<&<Self as Node>::Value> {
        self.node.lazy_value()
    }
}
impl<T> PersistentNode for PersistentWrapper<T>
where
    T: Node,
{
    fn left_child(&self) -> usize {
        self.left
    }

    fn right_child(&self) -> usize {
        self.right
    }

    fn set_children(&mut self, left: usize, right: usize) {
        self.left = left;
        self.right = right;
    }
}
