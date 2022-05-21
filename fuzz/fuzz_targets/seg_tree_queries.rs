use libfuzzer_sys::arbitrary::Arbitrary;
use seg_tree::nodes::Node;

pub enum queries<T: Node>{
    Query {
        i: usize,
        j: usize,
    },
    Update {
        i: usize,
        value: T
    }
}