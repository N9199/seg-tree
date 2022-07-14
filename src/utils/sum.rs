#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result, Unstructured};

use std::ops::{Add, Mul};

use crate::nodes::{LazyNode, Node};

/// Implementation of range sum for generic type T, it implements [Node] and [LazyNode], as such it can be used as a node in every segment tree type.
#[derive(Clone, Debug)]
pub struct Sum<T> {
    value: T,
    lazy_value: Option<T>,
}

#[cfg(feature = "arbitrary")]
impl<'a, T> Arbitrary<'a> for Sum<T>
where
    T: Arbitrary<'a>,
{
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let value = u.arbitrary()?;
        Ok(Self {
            value,
            lazy_value: None,
        })
    }
}

impl<T> Node for Sum<T>
where
    T: Add<Output = T> + Clone,
{
    type Value = T;
    /// The node is initialized with the value given.
    fn initialize(v: &Self::Value) -> Self {
        Sum {
            value: v.clone(),
            lazy_value: None,
        }
    }
    /// As this is a range sum node, the operation which is used to 'merge' two nodes is `+`.
    fn combine(a: &Self, b: &Self) -> Self {
        Sum {
            value: a.value.clone() + b.value.clone(),
            lazy_value: None,
        }
    }
    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// Implementation for sum range query node, the update adds the value to each item in the range.
/// It assumes that `a*n`, where a: T and n: usize is well defined and `a*n = a+...+a` with 'n' a.
/// For non-commutative operations, two things will be true `lazy_value = lazy_value + new_value`.
impl<T> LazyNode for Sum<T>
where
    T: Add<Output = T> + Mul<usize, Output = T> + Clone,
{
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

#[cfg(test)]
mod tests {
    use std::ops::{Add, Mul};

    use crate::{
        nodes::{LazyNode, Node},
        utils::Sum,
    };

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
        assert_eq!(result.value(), &500000500000);
    }

    #[test]
    fn non_commutative_sum_works() {
        let nodes: Vec<Sum<NonCommutativeTest>> = (0..=1000000)
            .map(|x| Sum::initialize(&NonCommutativeTest(x)))
            .collect();
        let result = nodes
            .iter()
            .fold(Sum::initialize(&NonCommutativeTest(0)), |acc, new| {
                Sum::combine(&acc, new)
            });
        assert_eq!(result.value(), &NonCommutativeTest(1000000));
    }

    #[test]
    fn update_lazy_value_works() {
        let mut node = Sum::initialize(&1);
        node.update_lazy_value(&2);
        assert_eq!(node.lazy_value(), Some(&2));
    }

    #[test]
    fn lazy_update_works() {
        // Node represents the range [0,10] with sum 1.
        let mut node = Sum::initialize(&1);
        node.update_lazy_value(&2);
        node.lazy_update(0, 10);
        assert_eq!(node.value(), &23);
    }

    #[test]
    fn non_commutative_update_lazy_value_works() {
        let mut node = Sum::initialize(&NonCommutativeTest(1));
        node.update_lazy_value(&NonCommutativeTest(2));
        assert_eq!(node.lazy_value(), Some(&NonCommutativeTest(2)));
    }
    #[test]
    fn non_commutative_lazy_update_works() {
        // Node represents the range [0,10] with sum 1.
        let mut node = Sum::initialize(&NonCommutativeTest(1));
        node.update_lazy_value(&NonCommutativeTest(2));
        node.lazy_update(0, 10);
        assert_eq!(node.value(), &NonCommutativeTest(2));
    }
}
