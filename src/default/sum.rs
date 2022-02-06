use std::ops::{Add, Mul};

use crate::nodes::{LazyNode, Node};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Sum<T>
where
    T: Add<Output = T> + Clone,
{
    value: T,
    lazy_value: Option<T>,
}

impl<T> Node for Sum<T>
where
    T: Add<Output = T> + Clone,
{
    type Value = T;
    fn initialize(v: Self::Value) -> Self {
        Sum {
            value: v,
            lazy_value: None,
        }
    }
    fn combine(a: &Self, b: &Self) -> Self {
        Sum {
            value: a.values() + b.values(),
            lazy_value: None,
        }
    }
    fn values(&self) -> Self::Value {
        self.value.clone()
    }
}

impl<T> LazyNode for Sum<T>
where
    T: Add<Output = T> + Mul<usize, Output = T> + Clone,
{
    /// Implementation for sum range query node, the update adds the value to each item in the range.
    /// It assumes that a*n, where a: T and n: usize is well defined and a*n = a+...+a with 'n' a.

    fn lazy_update(&mut self, v: &<Self as Node>::Value, i: usize, j: usize) {
        todo!()
    }

    fn update_lazy_value(&mut self, v: &<Self as Node>::Value) {
        if let Some(value) = &mut self.lazy_value {
            *value = value.clone() + v.clone();
        } else {
            self.lazy_value = Some(v.clone());
        }
    }

    fn lazy_value(&self) -> Option<<Self as Node>::Value> {
        self.lazy_value.clone()
    }
}

mod tests {
    use crate::nodes::Node;

    use super::Sum;

    #[test]
    fn sum_works() {
        let nodes: Vec<Sum<usize>> = (0..10).map(Sum::initialize).collect();
        let result = nodes
            .iter()
            .fold(Sum::initialize(0), |acc, new| Sum::combine(&acc, new));
        assert_eq!(result.values(), 55);
    }
    #[test]
    fn non_associative_sum_works() {
        todo!()
    }
    #[test]
    fn lazy_update_works() {
        todo!()
    }
    #[test]
    fn non_associative_lazy_update_works() {
        todo!()
    }
}
