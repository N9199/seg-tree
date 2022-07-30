use crate::nodes::Node;

/// Implementation of range max for generic type T, it only implements [`Node`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Max<T> {
    value: T,
}

impl<T> Node for Max<T>
where
    T: Ord + Clone,
{
    type Value = T;
    fn initialize(v: &Self::Value) -> Self {
        Self { value: v.clone() }
    }
    fn combine(a: &Self, b: &Self) -> Self {
        Self {
            value: a.value.clone().max(b.value.clone()),
        }
    }
    fn value(&self) -> &Self::Value {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use crate::{nodes::Node, utils::Max};

    #[test]
    fn max_works() {
        let nodes: Vec<Max<usize>> = (0..=1_000_000).map(|x| Max::initialize(&x)).collect();
        let result = nodes
            .iter()
            .fold(Max::initialize(&0), |acc, new| Max::combine(&acc, new));
        assert_eq!(result.value(), &1_000_000);
    }
}
