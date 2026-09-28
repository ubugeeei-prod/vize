//! Stable dynamic prop deduplication for generated patch flags.

use vize_l0::{FxHashSet, String};

/// Preserve the first occurrence of each name, including when duplicates are separated.
/// Small lists use the existing Vec as their scratch space; large lists retain the hash path.
pub(super) fn dedupe_dynamic_props(props: &mut Vec<String>) {
    if props.len() <= 8 {
        let mut index = 1;
        while index < props.len() {
            let duplicate = props.get(index).is_some_and(|current| {
                props
                    .get(..index)
                    .is_some_and(|seen| seen.contains(current))
            });
            if duplicate {
                props.remove(index);
            } else {
                index += 1;
            }
        }
    } else {
        let mut seen = FxHashSet::with_capacity_and_hasher(props.len(), Default::default());
        props.retain(|prop| seen.insert(prop.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_l0::ToCompactString;

    fn compact(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_compact_string()).collect()
    }

    #[test]
    fn keeps_first_occurrence_in_source_order() {
        let mut props = compact(&["class", "onClick", "style", "onClick", "class"]);
        dedupe_dynamic_props(&mut props);
        assert_eq!(props, compact(&["class", "onClick", "style"]));
    }

    #[test]
    fn small_and_large_paths_match_the_previous_hash_set_result() {
        for len in 0..=10 {
            for bits in 0..(1 << len) {
                let names: Vec<_> = (0..len)
                    .map(|index| if bits & (1 << index) == 0 { "a" } else { "b" })
                    .collect();
                let mut actual = compact(&names);
                let mut expected = actual.clone();
                let mut seen = FxHashSet::default();
                expected.retain(|prop| seen.insert(prop.clone()));
                dedupe_dynamic_props(&mut actual);
                assert_eq!(actual, expected, "len={len}, bits={bits}");
            }
        }
    }

    #[test]
    fn hundred_distinct_props_keep_order_when_a_late_duplicate_is_removed() {
        let mut props: Vec<String> = (0..100)
            .map(|index| vize_l0::cstr!("key-{index}"))
            .collect();
        let expected = props.clone();
        props.push(String::from("key-7"));
        dedupe_dynamic_props(&mut props);
        assert_eq!(props, expected);
    }
}
