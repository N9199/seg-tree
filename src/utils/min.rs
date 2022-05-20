use crate::nodes::{LazyNode, Node};

/// Implementation of range min for generic type T, it implements [Node] and [LazyNode].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Min<T>
where
    T: Ord + Clone,
{
    value: T,
    lazy_value: Option<T>,
}

impl<T> Node for Min<T>
where
    T: Ord + Clone,
{
    type Value = T;
    fn initialize(v: &Self::Value) -> Self {
        Min {
            value: v.clone(),
            lazy_value: None,
        }
    }
    fn combine(a: &Self, b: &Self) -> Self {
        Min {
            value: a.value.clone().min(b.value.clone()),
            lazy_value: None,
        }
    }
    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// Implementation for minimum range query, the update sets each item in the range to the given value.
impl<T> LazyNode for Min<T>
where
    T: Ord + Clone,
{
    fn lazy_update(&mut self, _i: usize, _j: usize) {
        if let Some(value) = self.lazy_value.take() {
            self.value = value
        }
    }

    fn update_lazy_value(&mut self, v: &<Self as Node>::Value) {
        self.lazy_value = Some(v.clone());
    }

    fn lazy_value(&self) -> Option<&<Self as Node>::Value> {
        self.lazy_value.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        nodes::{LazyNode, Node},
        utils::Min,
    };

    #[test]
    fn min_works() {
        let nodes: Vec<Min<usize>> = (0..=1000000).map(|x| Min::initialize(&x)).collect();
        let result = nodes
            .iter()
            .fold(Min::initialize(&0), |acc, new| Min::combine(&acc, new));
        assert_eq!(result.value(), &0);
    }

    #[test]
    fn update_lazy_value_works() {
        let mut node = Min::initialize(&1);
        node.update_lazy_value(&2);
        assert_eq!(node.lazy_value(), Some(&2));
    }

    #[test]
    fn lazy_update_works() {
        // Node represents the range [0,10] with min 1.
        let mut node = Min::initialize(&1);
        node.update_lazy_value(&2);
        node.lazy_update(0, 10);
        assert_eq!(node.value(), &2);
    }
}
