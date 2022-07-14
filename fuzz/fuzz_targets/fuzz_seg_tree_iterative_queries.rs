#![no_main]
use libfuzzer_sys::{arbitrary, fuzz_target};

use seg_tree::{nodes::Node, segment_tree::SegmentTree, utils::Max};

#[derive(arbitrary::Arbitrary, Debug)]
enum Query<T: Node> {
    Update { i: usize, value: T },
    Query { i: usize, j: usize },
}

type FuzzNode = Max<i64>;

fuzz_target!(|data: (Vec<FuzzNode>, Vec<Query<FuzzNode>>)| {
    let (base_data, queries) = data;
    let mut tree = SegmentTree::build(&base_data);
    let n = base_data.len();
    for q in queries {
        match q {
            Query::Update { i, value } => {
                if i < n {
                    tree.update(i, *value.value())
                }
            }
            Query::Query { mut i, mut j } => {
                if i > j {
                    core::mem::swap(&mut i, &mut j);
                }
                if j < n {
                    tree.query(i, j);
                }
            }
        }
    }
});
