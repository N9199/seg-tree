#![no_main]
use libfuzzer_sys::{arbitrary, fuzz_target};

use seg_tree::{
    nodes::Node,
    segment_tree::PersistentSegmentTree,
    utils::{Max, PersistentWrapper},
};

#[derive(arbitrary::Arbitrary, Debug)]
enum Query<T: Node> {
    Update { i: usize, v: usize, value: T },
    Query { i: usize, j: usize, v: usize },
}

type FuzzNode = PersistentWrapper<Max<i64>>;

fuzz_target!(|data: (Vec<FuzzNode>, Vec<Query<FuzzNode>>)| {
    let (base_data, queries) = data;
    let mut tree = PersistentSegmentTree::build(&base_data);
    let n = base_data.len();
    for q in queries {
        match q {
            Query::Update { i, v, value } => {
                if v < tree.versions() && i < n {
                    tree.update(v, i, *value.value())
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
