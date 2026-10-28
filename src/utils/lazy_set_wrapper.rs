use crate::traits::{LazyNode, Node};

/// A wrapper for nodes to easily implement [`LazyNode`] with an update which sets the range to a value.
#[derive(Clone)]
pub struct LazySetWrapper<T>
where
    T: Node,
{
    node: T,
    update: Option<<T as Node>::Value>,
}

impl<T> std::fmt::Debug for LazySetWrapper<T>
where
    T: Node + std::fmt::Debug,
    <T as Node>::Value: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazySetWrapper")
            .field("node", &self.node)
            .field("lazy_value", &self.update)
            .finish()
    }
}

impl<T> Node for LazySetWrapper<T>
where
    T: Node,
{
    type Value = <T as Node>::Value;

    #[inline]
    fn initialize(value: &Self::Value) -> Self {
        Self {
            node: Node::initialize(value),
            update: None,
        }
    }

    #[inline]
    fn combine(a: &Self, b: &Self) -> Self {
        Self {
            node: Node::combine(&a.node, &b.node),
            update: None,
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
    type Update = <T as Node>::Value;

    #[inline]
    fn pending(&self) -> Option<&Self::Update> {
        self.update.as_ref()
    }

    #[inline]
    fn defer(&mut self, update: &Self::Update) {
        self.update = Some(update.clone());
    }

    #[inline]
    fn apply_pending(&mut self, _range: std::range::RangeInclusive<usize>) {
        if let Some(value) = self.update.take() {
            self.node = Node::initialize(&value);
        }
    }
}
impl<T> From<T> for LazySetWrapper<T>
where
    T: Node,
{
    #[inline]
    fn from(node: T) -> Self {
        Self { node, update: None }
    }
}

#[cfg(test)]
mod tests {
    use std::range::RangeInclusive;

    use crate::{
        traits::{LazyNode, Node},
        utils::Min,
    };

    use super::LazySetWrapper;

    type LSMin<T: Clone + Ord> = LazySetWrapper<Min<T>>;
    #[test]
    fn update_lazy_value_works() {
        let mut node = LSMin::initialize(&1);
        node.defer(&2);
        assert_eq!(node.pending(), Some(&2));
    }

    #[test]
    fn lazy_update_works() {
        // Node represents the range [0,10] with min 1.
        let mut node = LSMin::initialize(&1);
        node.defer(&2);
        node.apply_pending(RangeInclusive { start: 0, last: 10 });
        assert_eq!(node.value(), &2);
    }
}
