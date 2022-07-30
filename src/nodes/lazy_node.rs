use super::Node;

/// Required trait by nodes of lazy segment trees.
/// It's defined as an interface for the operations needed on the `lazy_value`.
/// It is recommended to implement it using an Option type.
/// See [Implementors](LazyNode#implementors) for some example implementations.
pub trait LazyNode: Node {
    /// The following invariant must be met while implementing this method, if `lazy_value` is called immediately after this function then it must return `None`. (See [`Option::take`])
    fn lazy_update(&mut self, i: usize, j: usize);
    /// The following invariant must be met while implementing this method, if `lazy_value` is called immediately after this function then it must return `Some(&value)`.
    fn update_lazy_value(&mut self, new_value: &<Self as Node>::Value);
    /// Must return a reference to the current lazy value only if it exists.
    fn lazy_value(&self) -> Option<&<Self as Node>::Value>;
}
