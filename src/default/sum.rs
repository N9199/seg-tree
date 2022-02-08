use std::ops::{Add, Mul};

use crate::nodes::{LazyNode, Node};

#[derive(Clone)]
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
    fn initialize(v: &Self::Value) -> Self {
        Sum {
            value: v.clone(),
            lazy_value: None,
        }
    }
    fn combine(a: &Self, b: &Self) -> Self {
        Sum {
            value: a.value.clone() + b.value.clone(),
            lazy_value: None,
        }
    }
    fn values(&self) -> &Self::Value {
        &self.value
    }
}

impl<T> LazyNode for Sum<T>
where
    T: Add<Output = T> + Mul<usize, Output = T> + Clone,
{
    /// Implementation for sum range query node, the update adds the value to each item in the range.
    /// It assumes that a*n, where a: T and n: usize is well defined and a*n = a+...+a with 'n' a.
    /// For non-commutative operations, two things will be true lazy_value = lazy_value + new_value

    fn lazy_update(&mut self, i: usize, j: usize) {
        if let Some(value) = self.lazy_value.take() {
            let temp = self.value.clone() + value * (j - i + 1);
            self.value = temp;
        }
    }

    fn update_lazy_value(&mut self, new_value: &<Self as Node>::Value) {
        if let Some(value) = self.lazy_value.take() {
            self.lazy_value = Some(value + new_value.clone());
        } else {
            self.lazy_value = Some(new_value.clone());
        }
    }

    fn lazy_value(&self) -> Option<&<Self as Node>::Value> {
        self.lazy_value.as_ref()
    }
}

mod tests {
    use std::ops::{Add, Mul};

    use crate::{default::Sum, nodes::{Node, LazyNode}};

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct NonCommutativeTest(u64);
    /// It satisfies a+b==b
    impl Add for NonCommutativeTest {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            rhs
        }
    }

    impl Mul<usize> for NonCommutativeTest {
        type Output = Self;

        fn mul(self, _rhs: usize) -> Self::Output {
            self
        }
    }

    #[test]
    fn sum_works() {
        let nodes: Vec<Sum<usize>> = (0..=1000000).map(|x| Sum::initialize(&x)).collect();
        let result = nodes
            .iter()
            .fold(Sum::initialize(&0), |acc, new| Sum::combine(&acc, new));
        assert_eq!(result.values(), &500000500000);
    }
    #[test]
    fn non_commutative_sum_works() {
        let nodes: Vec<Sum<NonCommutativeTest>> = (0..=1000000).map(|x| Sum::initialize(&NonCommutativeTest(x))).collect();
        let result = nodes
            .iter()
            .fold(Sum::initialize(&NonCommutativeTest(0)), |acc, new| Sum::combine(&acc, new));
        assert_eq!(result.values(), &NonCommutativeTest(1000000));
    }
    #[test]
    fn update_lazy_value_works() {
        let mut node = Sum::initialize(&1);
        node.update_lazy_value(&2);
        assert_eq!(node.lazy_value(),Some(&2));
    }

    #[test]
    fn lazy_update_works() {
        // Node represents the range [0,10] with sum 1.
        let mut node = Sum::initialize(&1);
        node.update_lazy_value(&2);
        node.lazy_update(0, 10);
        assert_eq!(node.values(),&23);
    }

    #[test]
    fn non_commutative_update_lazy_value_works() {
        let mut node = Sum::initialize(&NonCommutativeTest(1));
        node.update_lazy_value(&NonCommutativeTest(2));
        assert_eq!(node.lazy_value(),Some(&NonCommutativeTest(2)));
    }
    #[test]
    fn non_commutative_lazy_update_works() {
        // Node represents the range [0,10] with sum 1.
        let mut node = Sum::initialize(&NonCommutativeTest(1));
        node.update_lazy_value(&NonCommutativeTest(2));
        node.lazy_update(0, 10);
        assert_eq!(node.values(),&NonCommutativeTest(2));
    }
}
