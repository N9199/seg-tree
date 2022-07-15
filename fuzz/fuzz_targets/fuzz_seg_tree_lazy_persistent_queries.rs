#![no_main]
use libfuzzer_sys::{arbitrary, fuzz_target};

use seg_tree::{
    nodes::Node,
    segment_tree::LazyPersistentSegmentTree,
    utils::{LazySetWrapper, Max, PersistentWrapper},
};

#[derive(arbitrary::Arbitrary, Debug)]
enum Query<T: Node> {
    Update {
        i: usize,
        j: usize,
        v: usize,
        value: T,
    },
    Query {
        i: usize,
        j: usize,
        v: usize,
    },
}

type FuzzNode = PersistentWrapper<LazySetWrapper<Max<i64>>>;

fuzz_target!(|data: (Vec<FuzzNode>, Vec<Query<FuzzNode>>)| {
    let (base_data, queries) = data;
    let mut tree = LazyPersistentSegmentTree::build(&base_data);
    let n = base_data.len();
    for q in queries {
        match q {
            Query::Update {
                mut i,
                mut j,
                v,
                value,
            } => {
                if i > j {
                    core::mem::swap(&mut i, &mut j);
                }
                if v < tree.versions() && j < n {
                    tree.update(v, i, j, *value.value())
                }
            }
            Query::Query { mut i, mut j, v } => {
                if i > j {
                    core::mem::swap(&mut i, &mut j);
                }
                if v < tree.versions() && j < n {
                    tree.query(v, i, j);
                }
            }
        }
    }
});
