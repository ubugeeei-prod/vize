//! Add the selected config without growing and re-sorting canonical watch lists.

use std::cmp::Ordering;
use vize_l0::String;

pub(super) fn include_root(paths: &mut Vec<String>, root: &str) {
    if dedup_if_ordered(paths) {
        if let Err(index) = paths.binary_search_by(|path| path.as_str().cmp(root)) {
            paths.insert(index, root.into());
        }
    } else {
        paths.push(root.into());
        paths.sort();
        paths.dedup();
    }
}

/// Deduplicate an ordered list with one comparison per name. Stop at the
/// first descending name: Pug includes can precede an otherwise sorted type
/// list. Until the whole list is known to be ordered, swaps retain every
/// original value, so the fallback can still sort and deduplicate them all.
fn dedup_if_ordered(paths: &mut Vec<String>) -> bool {
    let mut kept = usize::from(!paths.is_empty());
    for current in 1..paths.len() {
        let order = match (paths.get(kept - 1), paths.get(current)) {
            (Some(previous), Some(next)) => previous.cmp(next),
            _ => return false,
        };
        match order {
            Ordering::Greater => return false,
            Ordering::Equal => {}
            Ordering::Less => {
                if kept != current {
                    paths.swap(kept, current);
                }
                kept += 1;
            }
        }
    }
    paths.truncate(kept);
    true
}

#[cfg(test)]
mod tests {
    use super::include_root;
    use vize_l0::String;

    #[test]
    fn all_small_watch_lists_match_the_previous_normalization() {
        let names = ["/a.ts", "/tsconfig.json", "/z.ts"];
        for length in 0..=5 {
            for mut choice in 0..3usize.pow(length) {
                let mut paths: Vec<String> = (0..length)
                    .map(|_| {
                        let name = names.get(choice % names.len()).expect("base-three digit");
                        choice /= names.len();
                        String::from(*name)
                    })
                    .collect();
                let mut expected = paths.clone();
                expected.push("/tsconfig.json".into());
                expected.sort();
                expected.dedup();
                include_root(&mut paths, "/tsconfig.json");
                assert_eq!(paths, expected);
            }
        }
    }

    #[test]
    fn an_existing_config_keeps_the_canonical_list_storage() {
        let mut paths: Vec<String> = ["/project/src/types.ts", "/project/tsconfig.json"]
            .into_iter()
            .map(String::from)
            .collect();
        let allocation = paths.as_ptr();
        let capacity = paths.capacity();
        include_root(&mut paths, "/project/tsconfig.json");
        assert_eq!(paths.as_ptr(), allocation);
        assert_eq!(paths.capacity(), capacity);
        assert_eq!(paths.len(), 2);
    }
}
