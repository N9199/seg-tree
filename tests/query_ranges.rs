use seg_tree::{
    Iterative, LazyPersistent, LazyRecursive, Persistent, Recursive,
    traits::Node,
    utils::{LazySum, Sum},
};

fn assert_ranges_match(values: &[usize], mut query: impl FnMut(usize, usize) -> Option<usize>) {
    let mut endpoints: Vec<_> = (0..=values.len() + 2).collect();
    endpoints.extend([usize::MAX - 1, usize::MAX]);
    for &left in &endpoints {
        for &right in &endpoints {
            let expected = if left > right || left >= values.len() {
                None
            } else {
                Some(values[left..=right.min(values.len() - 1)].iter().sum())
            };
            assert_eq!(
                query(left, right),
                expected,
                "range [{left}, {right}], values {values:?}"
            );
        }
    }
}

#[test]
fn all_trees_intersect_queries_with_their_domain() {
    for n in [0, 1, 2, 3, 5, 8, 9] {
        let values: Vec<usize> = (1..=n).collect();
        let nodes: Vec<Sum<usize>> = values.iter().map(Sum::initialize).collect();
        let lazy_nodes: Vec<LazySum<usize>> = values.iter().map(LazySum::initialize).collect();
        let iterative = Iterative::build(&nodes);
        let recursive = Recursive::build(&nodes);
        let lazy_recursive = LazyRecursive::build(&lazy_nodes);
        let persistent = Persistent::build(&nodes);
        let mut lazy_persistent = LazyPersistent::build(&lazy_nodes);
        assert_ranges_match(&values, |l, r| {
            iterative.query(l, r).map(|node| *node.value())
        });
        assert_ranges_match(&values, |l, r| {
            recursive.query(l, r).map(|node| *node.value())
        });
        assert_ranges_match(&values, |l, r| {
            lazy_recursive.query(l, r).map(|node| *node.value())
        });
        assert_ranges_match(&values, |l, r| {
            persistent.query(0, l, r).map(|node| *node.value())
        });
        assert_ranges_match(&values, |l, r| {
            lazy_persistent.query(0, l, r).map(|node| *node.value())
        });
    }
}

#[test]
fn empty_persistent_trees_ignore_the_version_when_querying() {
    let tree = Persistent::<Sum<usize>>::build(&[]);
    let mut lazy_tree = LazyPersistent::<LazySum<usize>>::build(&[]);
    for version in [0, 1, usize::MAX] {
        assert_ranges_match(&[], |l, r| {
            tree.query(version, l, r).map(|node| *node.value())
        });
        assert_ranges_match(&[], |l, r| {
            lazy_tree.query(version, l, r).map(|node| *node.value())
        });
    }
}

#[test]
fn normalized_lazy_queries_respect_updates_and_versions() {
    let values = [1, 2, 3, 4, 5];
    let nodes: Vec<LazySum<usize>> = values.iter().map(LazySum::initialize).collect();
    let mut recursive = LazyRecursive::build(&nodes);
    let mut persistent = LazyPersistent::build(&nodes);
    recursive.update(0, 4, &10);
    persistent.update(0, 0, 4, &10);
    persistent.update(1, 1, 2, &3);
    assert_ranges_match(&[11, 12, 13, 14, 15], |l, r| {
        recursive.query(l, r).map(|node| *node.value())
    });
    for (version, expected) in [values, [11, 12, 13, 14, 15], [11, 15, 16, 14, 15]]
        .iter()
        .enumerate()
    {
        assert_ranges_match(expected, |l, r| {
            persistent.query(version, l, r).map(|node| *node.value())
        });
    }
}

#[test]
#[should_panic]
fn nonempty_persistent_tree_still_rejects_invalid_versions() {
    let tree = Persistent::build(&[Sum::initialize(&1usize)]);
    tree.query(usize::MAX, 1, 0);
}

#[test]
#[should_panic]
fn nonempty_lazy_persistent_tree_still_rejects_invalid_versions() {
    let mut tree = LazyPersistent::build(&[LazySum::initialize(&1usize)]);
    tree.query(usize::MAX, 1, 0);
}
