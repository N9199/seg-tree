use crate::nodes::{LazyNode, Node};

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
    fn initialize(v: Self::Value) -> Self {
        Min {
            value: v,
            lazy_value: None,
        }
    }
    fn combine(a: &Self, b: &Self) -> Self {
        Min {
            value: a.values().min(b.values()),
            lazy_value: None,
        }
    }
    fn values(&self) -> Self::Value {
        self.value.clone()
    }
}

/// Implementation for minimum range query, the update sets each item in the range to the given value.
impl<T> LazyNode for Min<T>
where
    T: Ord + Clone,
{
    fn lazy_update(&mut self, v: &<Self as Node>::Value, _i: usize, _j: usize) {
        self.value = v.clone();
        self.lazy_value = None;
    }

    fn update_lazy_value(&mut self, v: &<Self as Node>::Value) {
        self.lazy_value = Some(v.clone());
    }

    fn lazy_value(&self) -> Option<<Self as Node>::Value> {
        self.lazy_value.clone()
    }
}
