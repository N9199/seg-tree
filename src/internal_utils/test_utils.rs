use core::fmt::Debug;

/// Checks every nonempty inclusive range against an independent slice oracle.
/// The query adapter can capture a mutable tree and/or a persistent version.
/// Empty input performs no queries and imposes no empty-tree API policy.
#[track_caller]
pub fn assert_range_matches<Input: Debug, Output: Debug + PartialEq>(
    values: &[Input],
    mut query: impl FnMut(usize, usize) -> Output,
    mut expected: impl FnMut(&[Input]) -> Output,
) {
    for left in 0..values.len() {
        for right in left..values.len() {
            let expected = expected(&values[left..=right]);
            assert_eq!(
                query(left, right),
                expected,
                "range [{left}, {right}], values {values:?}"
            );
        }
    }
}

/// Checks first-matching, nonempty prefixes by scanning prefixes with a slice oracle.
/// Search adapters normalize their result to `Option<usize>`. APIs that cannot
/// represent a missing match should use targets with a matching prefix.
#[track_caller]
pub fn assert_prefix_search_matches<Input: Debug, Target: Debug>(
    values: &[Input],
    targets: impl IntoIterator<Item = Target>,
    mut search: impl FnMut(&Target) -> Option<usize>,
    mut matches: impl FnMut(&[Input], &Target) -> bool,
) {
    for target in targets {
        let expected = (0..values.len()).find(|&end| matches(&values[..=end], &target));
        assert_eq!(
            search(&target),
            expected,
            "prefix target {target:?}, values {values:?}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{assert_prefix_search_matches, assert_range_matches};

    #[test]
    fn range_helper_visits_every_range_with_a_mutable_adapter() {
        let values = [2, 5, 1];
        let mut calls = 0;
        assert_range_matches(
            &values,
            |left, right| {
                calls += 1;
                values[left..=right].iter().sum::<usize>()
            },
            |range| range.iter().sum::<usize>(),
        );
        assert_eq!(calls, 6);
    }

    #[test]
    fn range_helper_skips_empty_input() {
        assert_range_matches(
            &[] as &[usize],
            |_, _| -> usize { panic!("empty input has no nonempty ranges") },
            |_| 0,
        );
    }

    #[test]
    #[should_panic(expected = "range [0, 0]")]
    fn range_helper_detects_an_incorrect_result() {
        assert_range_matches(&[1], |_, _| 0, |range| range[0]);
    }

    #[test]
    fn prefix_helper_supports_non_numeric_targets_and_missing_matches() {
        let values = ["ab", "c", "d"];
        let answers = [Some(1), Some(2), None];
        let mut calls = 0;
        assert_prefix_search_matches(
            &values,
            ["abc", "abcd", "missing"],
            |_| {
                let result = answers[calls];
                calls += 1;
                result
            },
            |prefix, target| prefix.concat().starts_with(target),
        );
        assert_eq!(calls, 3);
    }

    #[test]
    fn prefix_helper_has_no_nonempty_prefix_for_empty_input() {
        assert_prefix_search_matches(
            &[] as &[usize],
            [0, 1],
            |_| None,
            |_, _| panic!("empty input has no nonempty prefixes"),
        );
    }

    #[test]
    #[should_panic(expected = "prefix target 3")]
    fn prefix_helper_detects_an_incorrect_result() {
        assert_prefix_search_matches(
            &[1, 2],
            [3],
            |_| Some(0),
            |prefix, target| prefix.iter().sum::<usize>() >= *target,
        );
    }
}
