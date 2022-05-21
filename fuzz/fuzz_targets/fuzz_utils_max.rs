#![no_main]
use libfuzzer_sys::fuzz_target;

use seg_tree::{utils::Max, nodes::Node};

fuzz_target!(|data: Vec<usize>| {
    let max = data.iter().fold(usize::MIN, |acc, x| acc.max(*x));
    let data = data
        .iter()
        .map(Max::initialize)
        .collect::<Vec<_>>();
    let result = data.iter().fold(Max::initialize(&usize::MIN), |acc, x|Max::combine(&acc, x));
    assert_eq!(max, *result.value());
});
