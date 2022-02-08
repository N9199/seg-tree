pub trait Node {
    type Value;

    fn initialize(value: &Self::Value) -> Self;
    fn combine(a: &Self, b: &Self) -> Self;
    fn values(&self) -> &Self::Value;
}

pub trait LazyNode: Node {
    /// The following invariant must be met while implementing this function, if lazy_value is called immediately after this function then it must return None.
    fn lazy_update(&mut self, i: usize, j: usize);
    /// The following invariant must be met while implementing this function, if lazy_value is called immediately after this function then it must return Some(value).
    fn update_lazy_value(&mut self, new_value: &<Self as Node>::Value);
    fn lazy_value(&self) -> Option<&<Self as Node>::Value>;
}

pub trait PersistentNode: Node {
    fn left(&self)->usize;
    fn right(&self)->usize;
    fn set_sons(&mut self, l: usize, r: usize);
}