#![allow(unused)]

use std::time::Duration;

use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use rand::{distributions::Uniform, prelude::Distribution, Rng};
use seg_tree::{
    nodes::Node,
    utils::{LazySetWrapper, Min},
    *,
};

type LSMin<T> = LazySetWrapper<Min<T>>;
const N: i64 = 1_000_000;

pub fn recursive_segment_tree_queries_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("recursive_segment_tree_queries_benchmark");
    let mut rng = rand::thread_rng();
    let node_distr = Uniform::from(-N..=N);
    for i in 1..=6 {
        for j in 1..10 {
            let n = j * 10_usize.pow(i);
            let nodes: Vec<_> = (&mut rng)
                .sample_iter(node_distr)
                .map(|x| Min::initialize(&x))
                .take(n)
                .collect();
            let segment_tree = Recursive::build(&nodes);
            let index_distr = Uniform::from(0..n);
            group.throughput(Throughput::Elements(n as u64));
            group.warm_up_time(Duration::from_secs(1));
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                b.iter_batched(
                    || {
                        Some((index_distr.sample(&mut rng), index_distr.sample(&mut rng)))
                            .map(|(i, j)| (i.min(j), i.max(j)))
                            .unwrap()
                    },
                    |(i, j)| segment_tree.query(i, j),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

pub fn lazy_recursive_segment_tree_queries_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("lazy_recursive_segment_tree_queries_benchmark");
    let mut rng = rand::thread_rng();
    let node_distr = Uniform::from(-N..=N);
    for i in 1..=6 {
        for j in 1..10 {
            let n = j * 10_usize.pow(i);
            let nodes: Vec<_> = (&mut rng)
                .sample_iter(node_distr)
                .map(|x| LSMin::initialize(&x))
                .take(n)
                .collect();
            let mut segment_tree = LazyRecursive::build(&nodes);
            let index_distr = Uniform::from(0..n);
            group.throughput(Throughput::Elements(n as u64));
            group.warm_up_time(Duration::from_secs(1));
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                b.iter_batched(
                    || {
                        Some((index_distr.sample(&mut rng), index_distr.sample(&mut rng)))
                            .map(|(i, j)| (i.min(j), i.max(j)))
                            .unwrap()
                    },
                    |(i, j)| segment_tree.query(i, j),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}
pub fn iterative_segment_tree_queries_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterative_segment_tree_queries_benchmark");
    let mut rng = rand::thread_rng();
    let node_distr = Uniform::from(-N..=N);
    for i in 1..=6 {
        for j in 1..10 {
            let n = j * 10_usize.pow(i);
            let nodes: Vec<_> = (&mut rng)
                .sample_iter(node_distr)
                .map(|x| Min::initialize(&x))
                .take(n)
                .collect();
            let segment_tree = Iterative::build(&nodes);
            let index_distr = Uniform::from(0..n);
            group.throughput(Throughput::Elements(n as u64));
            group.warm_up_time(Duration::from_secs(1));
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                b.iter_batched(
                    || {
                        Some((index_distr.sample(&mut rng), index_distr.sample(&mut rng)))
                            .map(|(i, j)| (i.min(j), i.max(j)))
                            .unwrap()
                    },
                    |(i, j)| segment_tree.query(i, j),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

pub fn iterative_segment_tree_updates_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterative_segment_tree_updates_benchmark");
    let mut rng = rand::thread_rng();
    let node_distr = Uniform::from(-N..=N);
    for i in 1..=6 {
        for j in 1..10 {
            let n = j * 10_usize.pow(i);
            let nodes: Vec<_> = (&mut rng)
                .sample_iter(node_distr)
                .map(|x| Min::initialize(&x))
                .take(n)
                .collect();
            let mut segment_tree = Iterative::build(&nodes);
            let index_distr = Uniform::from(0..n);
            group.throughput(Throughput::Elements(n as u64));
            group.warm_up_time(Duration::from_secs(1));
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                b.iter_batched(
                    || (index_distr.sample(&mut rng), node_distr.sample(&mut rng)),
                    |(i, v)| segment_tree.update(i, &v),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}
pub fn recursive_segment_tree_updates_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterative_segment_tree_updates_benchmark");
    let mut rng = rand::thread_rng();
    let node_distr = Uniform::from(-N..=N);
    for i in 1..=6 {
        for j in 1..10 {
            let n = j * 10_usize.pow(i);
            let nodes: Vec<_> = (&mut rng)
                .sample_iter(node_distr)
                .map(|x| Min::initialize(&x))
                .take(n)
                .collect();
            let mut segment_tree = Recursive::build(&nodes);
            let index_distr = Uniform::from(0..n);
            group.throughput(Throughput::Elements(n as u64));
            group.warm_up_time(Duration::from_secs(1));
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                b.iter_batched(
                    || (index_distr.sample(&mut rng), node_distr.sample(&mut rng)),
                    |(i, v)| segment_tree.update(i, &v),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}
pub fn lazy_recursive_segment_tree_updates_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterative_segment_tree_updates_benchmark");
    let mut rng = rand::thread_rng();
    let node_distr = Uniform::from(-N..=N);
    for i in 1..=6 {
        for j in 1..10 {
            let n = j * 10_usize.pow(i);
            let nodes: Vec<_> = (&mut rng)
                .sample_iter(node_distr)
                .map(|x| LSMin::initialize(&x))
                .take(n)
                .collect();
            let mut segment_tree = LazyRecursive::build(&nodes);
            let index_distr = Uniform::from(0..n);
            group.throughput(Throughput::Elements(n as u64));
            group.warm_up_time(Duration::from_secs(1));
            group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
                b.iter_batched(
                    || {
                        Some((
                            index_distr.sample(&mut rng),
                            index_distr.sample(&mut rng),
                            node_distr.sample(&mut rng),
                        ))
                        .map(|(i, j, v)| (i.min(j), i.max(j), v))
                        .unwrap()
                    },
                    |(i, j, v)| segment_tree.update(i, j, &v),
                    BatchSize::SmallInput,
                );
            });
        }
    }
    group.finish();
}

criterion_group!(
    benches,
    recursive_segment_tree_queries_benchmark,
    iterative_segment_tree_queries_benchmark,
    lazy_recursive_segment_tree_queries_benchmark,
    recursive_segment_tree_updates_benchmark,
    iterative_segment_tree_updates_benchmark
);
criterion_main!(benches);
