use super::Node;

pub trait WideNode<const B: usize>: Node + Sized {
    fn combine_multiple(values: &[Self; B]) -> Self;
}
