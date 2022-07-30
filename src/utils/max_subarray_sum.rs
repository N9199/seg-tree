use crate::nodes::Node;

/// Implementation of the solution to the maximum subarray problem. It just implements [`Node`].
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
        Self {
            max_sum: v,
            max_prefix_sum: v,
            max_suffix_sum: v,
            sum: v,
        }
    }
    fn combine(a: &Self, b: &Self) -> Self {
        Self {
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

#[cfg(test)]
mod tests {
    use rand::{prelude::SliceRandom, thread_rng};

    use crate::{nodes::Node, utils::MaxSubArraySum};

    #[test]
    fn max_sub_array_sum_works() {
        let mut rng = thread_rng();
        let n = 1_000_000 / 2;
        let mut nodes: Vec<_> = (-n..=n).collect();
        nodes.shuffle(&mut rng);
        // See https://en.wikipedia.org/wiki/Maximum_subarray_problem#Kadane's_algorithm
        let expected_answer = {
            let mut best_sum = 0;
            let mut current_sum = 0;
            for val in &nodes {
                current_sum = 0.max(current_sum + val);
                best_sum = best_sum.max(current_sum);
            }
            best_sum
        };
        let nodes: Vec<MaxSubArraySum> = nodes
            .into_iter()
            .map(|x| MaxSubArraySum::initialize(&x))
            .collect();
        let result = nodes
            .iter()
            .fold(MaxSubArraySum::initialize(&0), |acc, new| {
                MaxSubArraySum::combine(&acc, new)
            });
        assert_eq!(result.value(), &expected_answer);
    }
}
