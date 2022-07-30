use crate::nodes::{LazyNode, Node, PersistentNode};

/// A wrapper for nodes to easily implement [`LazyNode`] with an update which sets the range to a value. If the wrapped node implements [`PersistentNode`] the wrapper also implements it.
#[derive(Clone)]
pub struct LazySetWrapper<T>
where
    T: Node,
{
    node: T,
    lazy_value: Option<<T as Node>::Value>,
}

impl<T> std::fmt::Debug for LazySetWrapper<T>
where
    T: Node + std::fmt::Debug,
    <T as Node>::Value: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazySetWrapper")
            .field("node", &self.node)
            .field("lazy_value", &self.lazy_value)
            .finish()
    }
}

impl<T> Node for LazySetWrapper<T>
where
    T: Node,
{
    type Value = T::Value;

    #[inline]
    fn initialize(value: &Self::Value) -> Self {
        Self {
            node: T::initialize(value),
            lazy_value: None,
        }
    }

    #[inline]
    fn combine(a: &Self, b: &Self) -> Self {
        Self {
            node: T::combine(&a.node, &b.node),
            lazy_value: None,
        }
    }

    #[inline]
    fn value(&self) -> &Self::Value {
        self.node.value()
    }
}
impl<T> LazyNode for LazySetWrapper<T>
where
    T: Node,
{
    #[inline]
    fn lazy_update(&mut self, _i: usize, _j: usize) {
        if let Some(value) = self.lazy_value.take() {
            self.node = Node::initialize(&value);
        }
    }
    #[inline]
    fn update_lazy_value(&mut self, new_value: &<Self as Node>::Value) {
        self.lazy_value = Some(new_value.clone());
    }
    #[inline]
    fn lazy_value(&self) -> Option<&<Self as Node>::Value> {
        self.lazy_value.as_ref()
    }
}
impl<T> PersistentNode for LazySetWrapper<T>
where
    T: PersistentNode,
{
    #[inline]
    fn left_child(&self) -> usize {
        self.node.left_child()
    }
    #[inline]
    fn right_child(&self) -> usize {
        self.node.right_child()
    }
    #[inline]
    fn set_children(&mut self, left: usize, right: usize) {
        self.node.set_children(left, right);
    }
}

impl<T> From<T> for LazySetWrapper<T>
where
    T: Node,
{
    #[inline]
    fn from(node: T) -> Self {
        Self {
            node,
            lazy_value: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        nodes::{LazyNode, Node},
        utils::Min,
    };

    use super::LazySetWrapper;

    type LSMin<T> = LazySetWrapper<Min<T>>;
    #[test]
    fn update_lazy_value_works() {
        let mut node = LSMin::initialize(&1);
        node.update_lazy_value(&2);
        assert_eq!(node.lazy_value(), Some(&2));
    }

    #[test]
    fn lazy_update_works() {
        // Node represents the range [0,10] with min 1.
        let mut node = LSMin::initialize(&1);
        node.update_lazy_value(&2);
        node.lazy_update(0, 10);
        assert_eq!(node.value(), &2);
    }
}
