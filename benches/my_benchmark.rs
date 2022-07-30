#![allow(unused)]

use std::time::Duration;

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use rand::{distributions::Uniform, prelude::Distribution};
use seg_tree::{nodes::Node, *};
mod iterative {
    use seg_tree::nodes::Node;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct Min<T>
    where
        T: Ord + Clone,
    {
        value: T,
    }

    impl<T> Node for Min<T>
    where
        T: Ord + Clone,
    {
        type Value = T;
        fn initialize(v: &Self::Value) -> Self {
            Min { value: v.clone() }
        }
        fn combine(a: &Self, b: &Self) -> Self {
            Min {
                value: a.value.clone().min(b.value.clone()),
            }
        }
        fn value(&self) -> &Self::Value {
            &self.value
        }
    }
}
pub fn segment_tree_queries_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("segment_tree_queries_benchmark");
    for i in 1..=6 {
        for j in 1..10 {
            let n = j * 10_usize.pow(i);
            let nodes: Vec<_> = (0..=n).map(|x| iterative::Min::initialize(&x)).collect();
            let segment_tree = Iterative::build(&nodes);
            let distr = Uniform::from(0..n);
            let mut rng = rand::thread_rng();
            group.throughput(Throughput::Elements(n as u64));
            group.warm_up_time(Duration::from_secs(1));
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                b.iter_batched(
                    || distr.sample(&mut rng),
                    |mut i| segment_tree.query(i, i),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(benches, segment_tree_queries_benchmark);
criterion_main!(benches);
