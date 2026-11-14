use seg_tree::{
    Iterative, LazyPersistent, LazyRecursive, Persistent, Recursive,
    traits::Node,
    utils::{LazySum, Sum},
};

fn endpoints(n: usize) -> Vec<usize> {
    let mut endpoints: Vec<_> = (0..=n + 1).collect();
    endpoints.extend([usize::MAX - 1, usize::MAX]);
    endpoints
}

#[test]
fn point_updates_ignore_indices_outside_the_tree() {
    for n in [0, 1, 3, 5] {
        let values: Vec<usize> = (1..=n).collect();
        let nodes: Vec<Sum<usize>> = values.iter().map(Sum::initialize).collect();
        for index in endpoints(n) {
            let mut iterative = Iterative::build(&nodes);
            let mut recursive = Recursive::build(&nodes);
            let mut persistent = Persistent::build(&nodes);
            let mut expected = values.clone();
            if let Some(value) = expected.get_mut(index) {
                *value = 42;
            }
            iterative.update(index, &42);
            recursive.update(index, &42);
            // Invalid versions are ignored only when there is nothing to update.
            let version = if index < n { 0 } else { usize::MAX };
            persistent.update(version, index, &42);
            assert_eq!(
                persistent.versions(),
                usize::from(n > 0) + usize::from(index < n)
            );
            for (i, value) in expected.iter().enumerate() {
                assert_eq!(*iterative.query(i, i).unwrap().value(), *value);
                assert_eq!(*recursive.query(i, i).unwrap().value(), *value);
                let latest = persistent.versions() - 1;
                assert_eq!(*persistent.query(latest, i, i).unwrap().value(), *value);
                assert_eq!(*persistent.query(0, i, i).unwrap().value(), values[i]);
            }
        }
    }
}

#[test]
fn lazy_updates_intersect_ranges_and_skip_empty_intersections() {
    for n in [0, 1, 3, 5] {
        let values: Vec<usize> = (1..=n).collect();
        let nodes: Vec<LazySum<usize>> = values.iter().map(LazySum::initialize).collect();
        for left in endpoints(n) {
            for right in endpoints(n) {
                let mut recursive = LazyRecursive::build(&nodes);
                let mut persistent = LazyPersistent::build(&nodes);
                let mut expected = values.clone();
                let overlaps = left <= right && left < n;
                if overlaps {
                    for value in &mut expected[left..=right.min(n - 1)] {
                        *value += 10;
                    }
                }
                recursive.update(left, right, &10);
                let version = if overlaps { 0 } else { usize::MAX };
                persistent.update(version, left, right, &10);
                assert_eq!(
                    persistent.versions(),
                    usize::from(n > 0) + usize::from(overlaps)
                );
                for start in 0..n {
                    for end in start..n {
                        let expected_sum: usize = expected[start..=end].iter().sum();
                        assert_eq!(*recursive.query(start, end).unwrap().value(), expected_sum);
                        let latest = persistent.versions() - 1;
                        assert_eq!(
                            *persistent.query(latest, start, end).unwrap().value(),
                            expected_sum
                        );
                        let original_sum: usize = values[start..=end].iter().sum();
                        assert_eq!(
                            *persistent.query(0, start, end).unwrap().value(),
                            original_sum
                        );
                    }
                }
            }
        }
    }
}
