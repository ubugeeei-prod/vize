//! `baseUrl` emulation for the generated tsconfig (#3886).
//!
//! The native checker removed `baseUrl`, so [`super`] strips it from the
//! generated config — but TypeScript 5.x/6.x projects rely on it to resolve
//! *bare* specifiers (`import "src/base/greet"`) relative to the baseUrl
//! directory. Stripping it without a replacement turned every such import into
//! a false `TS2307` while `vue-tsc` resolved it fine.
//!
//! The faithful replacement is the one the checker's own removal message
//! recommends: a `"*"` `paths` entry targeting `<baseUrl>/*`. TypeScript tries
//! `paths` patterns most-specific-first and falls back to ordinary
//! `node_modules` resolution when a matched target does not exist, so the
//! wildcard cannot shadow either the user's own aliases or package imports.

use serde_json::{Map, Value};
use vize_carton::cstr;

/// Insert the synthesized `"*"` alias for `base_url` into the captured `paths`
/// map, before the map is remapped into mirror/real-tree candidates.
///
/// `base_url` arrives the way rebased `paths` targets are spelled: relative to
/// the project root without a `./` prefix (empty for the root itself), or
/// absolute when it escapes the root. A user-declared `"*"` entry wins — it is
/// what TypeScript itself would consult first, and second-guessing it would
/// change resolution order.
#[allow(clippy::disallowed_types)]
pub(super) fn insert_wildcard_alias(
    paths: &mut Map<std::string::String, Value>,
    base_url: Option<&str>,
) {
    let Some(base_url) = base_url else {
        return;
    };
    if paths.contains_key("*") {
        return;
    }
    let target = if base_url.is_empty() {
        cstr!("*")
    } else {
        cstr!("{base_url}/*")
    };
    paths.insert("*".into(), Value::Array(vec![Value::String(target.into())]));
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    fn paths_with(entries: &[(&str, Value)]) -> Map<std::string::String, Value> {
        let mut paths = Map::new();
        for (alias, targets) in entries {
            paths.insert((*alias).into(), targets.clone());
        }
        paths
    }

    #[test]
    fn a_root_base_url_synthesizes_a_bare_wildcard() {
        let mut paths = paths_with(&[]);
        super::insert_wildcard_alias(&mut paths, Some(""));
        assert_eq!(paths.get("*"), Some(&json!(["*"])));
    }

    #[test]
    fn a_nested_base_url_prefixes_the_wildcard_target() {
        let mut paths = paths_with(&[("@/*", json!(["src/*"]))]);
        super::insert_wildcard_alias(&mut paths, Some("src"));
        assert_eq!(paths.get("*"), Some(&json!(["src/*"])));
        assert_eq!(
            paths.get("@/*"),
            Some(&json!(["src/*"])),
            "user aliases untouched"
        );
    }

    #[test]
    fn an_escaping_base_url_keeps_its_absolute_target() {
        let mut paths = paths_with(&[]);
        super::insert_wildcard_alias(&mut paths, Some("/outside/root"));
        assert_eq!(paths.get("*"), Some(&json!(["/outside/root/*"])));
    }

    #[test]
    fn a_user_declared_wildcard_wins() {
        let mut paths = paths_with(&[("*", json!(["vendor/*"]))]);
        super::insert_wildcard_alias(&mut paths, Some("src"));
        assert_eq!(paths.get("*"), Some(&json!(["vendor/*"])));
    }

    #[test]
    fn no_base_url_synthesizes_nothing() {
        let mut paths = paths_with(&[]);
        super::insert_wildcard_alias(&mut paths, None);
        assert!(paths.is_empty());
    }
}
