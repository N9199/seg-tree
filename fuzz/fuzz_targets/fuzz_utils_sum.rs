#![no_main]
use libfuzzer_sys::fuzz_target;

use seg_tree::{utils::Sum, nodes::Node};

fuzz_target!(|data: Vec<usize>| {
    if let Some(sum) = data.iter().fold(Some(0), |acc, x| {
        (*x).checked_add(acc?)
    }) {
        let data = data
            .iter()
            .map(Sum::initialize)
            .collect::<Vec<_>>();
        let result = data.iter().fold(Sum::initialize(&0), |acc, x|Sum::combine(&acc, x));
        assert_eq!(sum, *result.value());
    }
});
