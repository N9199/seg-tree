use crate::traits::{LazyNode, Node};

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

/// A range-minimum node with lazy updates that assign a value to every element.
#[derive(Clone, Debug)]
pub struct LazyMin<T> {
    value: T,
    update: Option<T>,
}

impl<T: Ord + Clone> Node for LazyMin<T> {
    type Value = T;

    fn initialize(value: &T) -> Self {
        Self {
            value: value.clone(),
            update: None,
        }
    }

    fn combine(left: &Self, right: &Self) -> Self {
        Self::initialize(std::cmp::min(&left.value, &right.value))
    }

    fn value(&self) -> &T {
        &self.value
    }
}

impl<T: Ord + Clone> LazyNode for LazyMin<T> {
    type Update = T;

    fn pending(&self) -> Option<&T> {
        self.update.as_ref()
    }

    fn defer(&mut self, update: &T) {
        self.update = Some(update.clone());
    }

    fn apply_pending(&mut self, _range: core::range::RangeInclusive<usize>) {
        if let Some(value) = self.update.take() {
            self.value = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        traits::{LazyNode, Node},
        utils::{LazyMin, Min},
    };

    #[test]
    fn assignment_uses_last_update_for_any_segment_length() {
        for range in [7..=7, 7..=11] {
            let mut node = LazyMin::initialize(&1);
            node.defer(&2);
            node.defer(&3);
            assert_eq!(*node.value(), 1);
            assert_eq!(node.pending(), Some(&3));
            node.apply_pending(core::range::RangeInclusive {
                start: *range.start(),
                last: *range.end(),
            });
            assert_eq!(*node.value(), 3);
            assert!(node.pending().is_none());
            node.apply_pending(core::range::RangeInclusive { start: 7, last: 7 });
            assert_eq!(*node.value(), 3);
        }
    }

    #[test]
    fn min_works() {
        const N: usize = 1_000;
        let nodes: Vec<Min<usize>> = (0..=N).map(|x| Min::initialize(&x)).collect();
        let result = nodes
            .iter()
            .fold(Min::initialize(&0), |acc, new| Min::combine(&acc, new));
        assert_eq!(result.value(), &0);
    }
}
