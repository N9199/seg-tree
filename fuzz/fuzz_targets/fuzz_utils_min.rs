#![no_main]
use libfuzzer_sys::fuzz_target;

use seg_tree::{utils::Min, nodes::Node};

fuzz_target!(|data: Vec<usize>| {
    let min = data.iter().fold(usize::MAX, |acc, x| acc.min(*x));
    let data = data
        .iter()
        .map(Min::initialize)
        .collect::<Vec<_>>();
    let result = data.iter().fold(Min::initialize(&usize::MAX), |acc, x|Min::combine(&acc, x));
    assert_eq!(min, *result.value());
});
