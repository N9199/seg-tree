use crate::nodes::Node;


/// Implementation of the solution to the maximum subarray problem. It just implements [Node].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MaxSubArraySum {
    max_sum: i64,
    max_prefix_sum: i64,
    max_suffix_sum: i64,
    sum: i64,
}

impl Node for MaxSubArraySum {
    type Value = i64;
    fn initialize(value: &Self::Value) -> Self {
        let v = value.to_owned();
        MaxSubArraySum {
            max_sum: v,
            max_prefix_sum: v,
            max_suffix_sum: v,
            sum: v,
        }
    }
    fn combine(a: &Self, b: &Self) -> Self {
        MaxSubArraySum {
            max_sum: a
                .max_sum
                .max(b.max_sum)
                .max(a.max_suffix_sum + b.max_prefix_sum),
            max_prefix_sum: a.max_prefix_sum.max(a.sum + b.max_prefix_sum),
            max_suffix_sum: b.max_suffix_sum.max(b.sum + a.max_suffix_sum),
            sum: a.sum + b.sum,
        }
    }
    fn value(&self) -> &Self::Value {
        &self.max_sum
    }
}
