use vize_carton::{String, ToCompactString};

/// Outcome of an `exports` lookup for one subpath key.
pub(super) enum ExportsTypes {
    /// The `types` target published for the subpath.
    Types(String),
    /// The subpath matched a `null` target. Node treats those as explicitly
    /// blocked, so resolution must stop instead of falling back to a path
    /// lookup that would expose the declaration anyway.
    Excluded,
}

/// Find the `types` condition for an `exports` subpath entry. An exact key
/// wins; otherwise a `*` pattern key is matched and the captured segment is
/// substituted into its target, so wildcard exports still yield declarations.
pub(super) fn exports_types_entry(manifest: &serde_json::Value, key: &str) -> Option<ExportsTypes> {
    let exports = manifest.get("exports")?;
    if let Some(entry) = exports.get(key) {
        if entry.is_null() {
            return Some(ExportsTypes::Excluded);
        }
        // An exact key ends resolution the way Node does, so an entry without a
        // `types` condition must not borrow declarations from a wildcard key.
        return super::find_types_condition(entry).map(ExportsTypes::Types);
    }
    let (captured, target) = best_pattern_match(exports, key)?;
    if target.is_null() {
        return Some(ExportsTypes::Excluded);
    }
    Some(ExportsTypes::Types(
        super::find_types_condition(target)?
            .replace('*', captured)
            .to_compact_string(),
    ))
}

/// Match `key` against Node-style `exports` pattern keys such as `"./dist/*"`,
/// returning the captured wildcard segment and the pattern's target value.
///
/// The most specific pattern wins: longest literal prefix first, then longest
/// literal suffix, matching Node's package export resolution order.
pub(super) fn best_pattern_match<'a>(
    exports: &'a serde_json::Value,
    key: &'a str,
) -> Option<(&'a str, &'a serde_json::Value)> {
    let mut best: Option<(usize, usize, &str, &serde_json::Value)> = None;
    for (pattern, target) in exports.as_object()? {
        if !pattern.starts_with("./") {
            continue;
        }
        let Some((prefix, suffix)) = pattern.split_once('*') else {
            continue;
        };
        let Some(captured) = key
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(suffix))
        else {
            continue;
        };
        if best.is_none_or(|(best_prefix, best_suffix, _, _)| {
            (prefix.len(), suffix.len()) > (best_prefix, best_suffix)
        }) {
            best = Some((prefix.len(), suffix.len(), captured, target));
        }
    }
    best.map(|(_, _, captured, target)| (captured, target))
}

#[cfg(test)]
mod tests {
    use super::{ExportsTypes, best_pattern_match, exports_types_entry};

    fn exports(raw: &str) -> serde_json::Value {
        serde_json::from_str(raw).unwrap()
    }

    #[test]
    fn captures_the_wildcard_segment() {
        let exports = exports(r#"{"./dist/*":{"types":"./types/*.d.ts"}}"#);
        let (captured, target) = best_pattern_match(&exports, "./dist/nested/foo").unwrap();
        assert_eq!(captured, "nested/foo");
        assert_eq!(target["types"], "./types/*.d.ts");
    }

    #[test]
    fn prefers_the_most_specific_pattern() {
        let exports =
            exports(r#"{"./*":{"types":"./root/*.d.ts"},"./dist/*.js":{"types":"./dist/*.d.ts"}}"#);
        let (captured, target) = best_pattern_match(&exports, "./dist/foo.js").unwrap();
        assert_eq!(captured, "foo");
        assert_eq!(target["types"], "./dist/*.d.ts");
    }

    #[test]
    fn ignores_condition_keys_and_unmatched_patterns() {
        let exports = exports(r#"{"types":"./root.d.ts","./dist/*":{"types":"./dist/*.d.ts"}}"#);
        assert!(best_pattern_match(&exports, "./other/foo").is_none());
    }

    #[test]
    fn a_more_specific_null_pattern_blocks_the_subpath() {
        let manifest =
            exports(r#"{"exports":{"./*":{"types":"./types/*.d.ts"},"./private/*":null}}"#);
        assert!(matches!(
            exports_types_entry(&manifest, "./private/secret"),
            Some(ExportsTypes::Excluded)
        ));
        assert!(matches!(
            exports_types_entry(&manifest, "./button"),
            Some(ExportsTypes::Types(types)) if types == "./types/button.d.ts"
        ));
    }

    #[test]
    fn an_exact_null_key_blocks_the_subpath() {
        let manifest = exports(r#"{"exports":{"./*":{"types":"./types/*.d.ts"},"./secret":null}}"#);
        assert!(matches!(
            exports_types_entry(&manifest, "./secret"),
            Some(ExportsTypes::Excluded)
        ));
    }

    #[test]
    fn an_exact_key_without_types_does_not_borrow_from_a_wildcard() {
        let manifest = exports(
            r#"{"exports":{"./style.css":"./dist/style.css","./*":{"types":"./types/*.d.ts"}}}"#,
        );
        assert!(exports_types_entry(&manifest, "./style.css").is_none());
    }

    #[test]
    fn an_unexported_subpath_stays_unmatched() {
        let manifest = exports(r#"{"exports":{"./dist/*":{"types":"./dist/*.d.ts"}}}"#);
        assert!(exports_types_entry(&manifest, "./other").is_none());
    }
}
