use core::num::NonZeroUsize;

use crate::nodes::{LazyNode, Node};

#[derive(Clone, Copy)]
pub struct NonNUsize<const N: usize>(NonZeroUsize);

impl<const N: usize> NonNUsize<N> {
    pub fn new(n: usize) -> Option<Self> {
        NonZeroUsize::new(n ^ N).map(NonNUsize)
    }

    pub const fn get(self) -> usize {
        self.0.get() ^ N
    }
}

impl<const N: usize> std::fmt::Debug for NonNUsize<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0.get() ^ N))
    }
}
type IUsize = NonNUsize<{ usize::MAX }>;

#[derive(Clone, Debug)]
pub struct PersistentWrapper<T> {
    node: T,
    left: Option<IUsize>,
    right: Option<IUsize>,
}

impl<T> Node for PersistentWrapper<T>
where
    T: Node,
{
    type Value = T::Value;

    #[inline]
    fn initialize(value: &Self::Value) -> Self {
        Self {
            node: T::initialize(value),
            left: None,
            right: None,
        }
    }

    #[inline]
    fn combine(a: &Self, b: &Self) -> Self {
        Self {
            node: T::combine(&a.node, &b.node),
            left: None,
            right: None,
        }
    }
    #[inline]
    fn value(&self) -> &Self::Value {
        self.node.value()
    }
}
impl<T> LazyNode for PersistentWrapper<T>
where
    T: LazyNode,
{
    #[inline]
    fn lazy_update(&mut self, i: usize, j: usize) {
        self.node.lazy_update(i, j);
    }

    #[inline]
    fn update_lazy_value(&mut self, new_value: &<Self as Node>::Value) {
        self.node.update_lazy_value(new_value);
    }

    #[inline]
    fn lazy_value(&self) -> Option<&<Self as Node>::Value> {
        self.node.lazy_value()
    }
}

impl<T> From<T> for PersistentWrapper<T>
where
    T: Node,
{
    #[inline]
    fn from(node: T) -> Self {
        Self {
            node,
            left: None,
            right: None,
        }
    }
}

impl<T> PersistentWrapper<T> {
    #[inline]
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_inner(self) -> T {
        self.node
    }

    #[inline]
    pub const fn left_child(&self) -> Option<IUsize> {
        self.left
    }
    #[inline]
    pub const fn right_child(&self) -> Option<IUsize> {
        self.right
    }

    #[inline]
    pub fn set_children(&mut self, left: usize, right: usize) {
        self.right = IUsize::new(right);
        self.left = IUsize::new(left);
    }

    #[inline]
    pub const fn get_inner(&self) -> &T {
        &self.node
    }
}

#[cfg(test)]
mod test {
    use super::NonNUsize;

    #[test]
    fn non_n_works() {
        let test = NonNUsize::<1>::new(0);
        assert_eq!(test.unwrap().get(), 0);
    }

    #[test]
    fn non_n_works2() {
        let test = NonNUsize::<1>::new(1);
        assert!(test.is_none());
    }
}
