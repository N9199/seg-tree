use crate::nodes::Node;

/// Implementation of range min for generic type T, it only implements [`Node`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Min<T> {
    value: T,
}

impl<T> Node for Min<T>
where
    T: Ord + Clone,
{
    type Value = T;
    fn initialize(v: &Self::Value) -> Self {
        Self { value: v.clone() }
    }
    fn combine(a: &Self, b: &Self) -> Self {
        Self {
            value: a.value.clone().min(b.value.clone()),
        }
    }
    fn value(&self) -> &Self::Value {
        &self.value
    }
}
#[cfg(test)]
mod tests {
    use crate::{nodes::Node, utils::Min};

    #[test]
    fn min_works() {
        let nodes: Vec<Min<usize>> = (0..=1_000_000).map(|x| Min::initialize(&x)).collect();
        let result = nodes
            .iter()
            .fold(Min::initialize(&0), |acc, new| Min::combine(&acc, new));
        assert_eq!(result.value(), &0);
    }
}
