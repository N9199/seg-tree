/// Base trait required by nodes of segment trees
/// A node which implements this trait is expected to have a way to its value, which must be of type Value.
pub trait Node {
    /// Type of value saved in node.
    type Value;

    fn initialize(value: &Self::Value) -> Self;
    fn combine(a: &Self, b: &Self) -> Self;
    fn value(&self) -> &Self::Value;
}


/// Required trait by nodes of lazy segment trees.
/// A node which implements this trait is expected to have a way to save its lazy_value.
pub trait LazyNode: Node {
    /// The following invariant must be met while implementing this method, if lazy_value is called immediately after this function then it must return None.
    fn lazy_update(&mut self, i: usize, j: usize);
    /// The following invariant must be met while implementing this method, if lazy_value is called immediately after this function then it must return Some(value).
    fn update_lazy_value(&mut self, new_value: &<Self as Node>::Value);
    fn lazy_value(&self) -> Option<&<Self as Node>::Value>;
}

/// Required trait by nodes of persistent segment trees.
/// A node which implements this trait is expected to have a way to save the indices of its sons. 
pub trait PersistentNode: Node {
    /// Gives index of left son.
    fn left(&self) -> usize;
    /// Gives index of right son.
    fn right(&self) -> usize;
    /// Sets saved index of both left and right sons.
    fn set_sons(&mut self, left: usize, right: usize);

    /// Default implementation assumes that only a leaf the index of the leaf sons are the same.
    fn is_leaf(&self) -> bool {
        self.left() == self.right()
    }
}
