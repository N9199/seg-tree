use super::Node;
/// Required trait by nodes of persistent segment trees.
pub trait PersistentNode: Node {
    /// Gives index of left child.
    fn left_child(&self) -> usize;
    /// Gives index of right child.
    fn right_child(&self) -> usize;
    /// Sets saved index of both left and right children.
    fn set_children(&mut self, left: usize, right: usize);
}
