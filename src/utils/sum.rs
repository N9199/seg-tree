use std::ops::{Add, Mul};

use crate::traits::{LazyNode, Node};

/// Implementation of range sum for generic type T, it implements [`Node`].
#[derive(Clone, Debug)]
pub struct Sum<T> {
    value: T,
}

impl<T> Node for Sum<T>
where
    T: Add<Output = T> + Clone,
{
    type Value = T;
    /// The node is initialized with the value given.
    #[inline]
    fn initialize(v: &Self::Value) -> Self {
        Self { value: v.clone() }
    }
    /// As this is a range sum node, the operation which is used to 'merge' two nodes is `+`.
    #[inline]
    fn combine(a: &Self, b: &Self) -> Self {
        Self {
            value: a.value.clone() + b.value.clone(),
        }
    }
    #[inline]
    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// A range-sum node with lazy updates that add a value to every element in a range.
#[derive(Debug, Clone)]
pub struct LazySum<T> {
    value: T,
    update: Option<T>,
}

impl<T> Node for LazySum<T>
where
    T: Add<Output = T> + Clone,
{
    type Value = T;
    /// The node is initialized with the value given.
    #[inline]
    fn initialize(v: &Self::Value) -> Self {
        Self {
            value: v.clone(),
            update: None,
        }
    }
    /// As this is a range sum node, the operation which is used to 'merge' two nodes is `+`.
    #[inline]
    fn combine(a: &Self, b: &Self) -> Self {
        Self {
            value: a.value.clone() + b.value.clone(),
            update: None,
        }
    }
    #[inline]
    fn value(&self) -> &Self::Value {
        &self.value
    }
}
/// Implementation for sum range query node, the update adds the value to each item in the range.
/// It assumes that `a*n`, where a: T and n: usize is well defined and `a*n = a+...+a` with 'n' a.
/// Pending additions are composed as `existing + incoming`.
impl<T> LazyNode for LazySum<T>
where
    T: Add<Output = T> + Mul<usize, Output = T> + Clone,
{
    type Update = <Sum<T> as Node>::Value;

    #[inline]
    fn pending(&self) -> Option<&Self::Update> {
        self.update.as_ref()
    }

    #[inline]
    fn defer(&mut self, update: &Self::Update) {
        if let Some(value) = self.update.take() {
            self.update = Some(value + update.clone());
        } else {
            self.update = Some(update.clone());
        }
    }

    #[inline]
    fn apply_pending(&mut self, range: std::range::RangeInclusive<usize>) {
        if let Some(value) = self.update.take() {
            let temp = self.value.clone() + value * (range.last - range.start + 1);
            self.value = temp;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::{Add, Mul},
        range::RangeInclusive,
    };

    use crate::{
        traits::{LazyNode, Node},
        utils::{Sum, sum::LazySum},
    };

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct NonCommutativeTest(usize);
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

    const N: usize = 1_000;

    #[test]
    fn sum_works() {
        let nodes: Vec<Sum<usize>> = (0..=N).map(|x| Sum::initialize(&x)).collect();
        let result = nodes
            .iter()
            .fold(Sum::initialize(&0), |acc, new| Sum::combine(&acc, new));
        assert_eq!(result.value(), &((N + 1) * N / 2));
    }

    #[test]
    fn non_commutative_sum_works() {
        let nodes: Vec<Sum<NonCommutativeTest>> = (0..=N)
            .map(|x| Sum::initialize(&NonCommutativeTest(x)))
            .collect();
        let result = nodes
            .iter()
            .fold(Sum::initialize(&NonCommutativeTest(0)), |acc, new| {
                Sum::combine(&acc, new)
            });
        assert_eq!(result.value(), &NonCommutativeTest(N));
    }

    #[test]
    fn update_lazy_value_works() {
        let mut node = LazySum::initialize(&1);
        node.defer(&2);
        assert_eq!(node.pending(), Some(&2));
    }

    #[test]
    fn lazy_update_works() {
        // Node represents the range [0,10] with sum 1.
        let mut node = LazySum::initialize(&1);
        node.defer(&2);
        node.apply_pending(RangeInclusive { start: 0, last: 10 });
        assert_eq!(node.value(), &23);
    }

    #[test]
    fn non_commutative_update_lazy_value_works() {
        let mut node = LazySum::initialize(&NonCommutativeTest(1));
        node.defer(&NonCommutativeTest(2));
        assert_eq!(node.pending(), Some(&NonCommutativeTest(2)));
    }
    #[test]
    fn non_commutative_lazy_update_works() {
        // Node represents the range [0,10] with sum 1.
        let mut node = LazySum::initialize(&NonCommutativeTest(1));
        node.defer(&NonCommutativeTest(2));
        node.apply_pending(RangeInclusive { start: 0, last: 10 });
        assert_eq!(node.value(), &NonCommutativeTest(2));
    }
}
