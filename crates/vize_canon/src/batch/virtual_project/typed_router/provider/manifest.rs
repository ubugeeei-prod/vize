//! Bounded package-export proof preserves the manifest's actual member order.

use serde::Deserialize;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use vize_carton::{FxHashSet, String, cstr};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Refusal {
    pub(super) code: &'static str,
    pub(super) detail: String,
}

impl Refusal {
    pub(super) fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

/// This intentionally does not deserialize objects into serde_json::Map:
/// its default backing map sorts keys and cannot prove condition precedence.
#[derive(Debug)]
enum OrderedValue {
    Object(Vec<(String, Self)>),
    Array,
    Text(String),
    Other,
}

impl<'de> Deserialize<'de> for OrderedValue {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OrderedVisitor;
        impl<'de> Visitor<'de> for OrderedVisitor {
            type Value = OrderedValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an ordered JSON value")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                let mut names = FxHashSet::default();
                while let Some((name, value)) = access.next_entry::<String, OrderedValue>()? {
                    if !names.insert(name.clone()) {
                        return Err(de::Error::custom(cstr!("duplicate JSON member '{name}'")));
                    }
                    entries.push((name, value));
                }
                Ok(OrderedValue::Object(entries))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                while access.next_element::<de::IgnoredAny>()?.is_some() {}
                Ok(OrderedValue::Array)
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(OrderedValue::Text(value.into()))
            }

            fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
                Ok(OrderedValue::Other)
            }

            fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
                Ok(OrderedValue::Other)
            }

            fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
                Ok(OrderedValue::Other)
            }

            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
                Ok(OrderedValue::Other)
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(OrderedValue::Other)
            }
        }
        deserializer.deserialize_any(OrderedVisitor)
    }
}

fn member<'a>(value: &'a OrderedValue, name: &str) -> Option<&'a OrderedValue> {
    let OrderedValue::Object(entries) = value else {
        return None;
    };
    entries
        .iter()
        .find_map(|(key, value)| (key == name).then_some(value))
}

/// Prove an explicit declaration target for one real active condition set.
/// The caller must obtain every possible context from the selected compiler
/// options and require one identical winner before admitting a project.
pub(super) fn experimental_type_target(
    source: &str,
    active_conditions: &[&str],
) -> Result<String, Refusal> {
    let manifest: OrderedValue = serde_json::from_str(source).map_err(|error| {
        Refusal::new(
            "invalid-package-json",
            cstr!("cannot prove package exports: {error}"),
        )
    })?;
    let exports = member(&manifest, "exports").ok_or_else(|| {
        Refusal::new("missing-package-exports", "package exports are unavailable")
    })?;
    let OrderedValue::Object(subpaths) = exports else {
        return Err(Refusal::new(
            "unsupported-exports-root",
            "an explicit subpath exports object is required",
        ));
    };
    if subpaths
        .iter()
        .any(|(key, _)| key != "." && !key.starts_with("./"))
    {
        return Err(Refusal::new(
            "mixed-exports-root",
            "package exports mix subpaths with condition or invalid keys",
        ));
    }
    let entry = member(exports, "./experimental").ok_or_else(|| {
        Refusal::new(
            "missing-experimental-entry",
            "an explicit experimental subpath is required",
        )
    })?;
    let target = select(entry, active_conditions, false)?
        .ok_or_else(|| Refusal::new("no-condition-winner", "no explicit type target wins"))?;
    if !target.starts_with("./")
        || target.split('/').skip(1).any(|part| {
            part.is_empty()
                || matches!(part, "." | "..")
                || part.eq_ignore_ascii_case("node_modules")
        })
        || target.contains(['\\', '?', '#', '%'])
        || target.chars().any(char::is_control)
        || ![".d.ts", ".d.mts", ".d.cts"]
            .iter()
            .any(|suffix| target.ends_with(suffix))
    {
        return Err(Refusal::new(
            "unsupported-type-target",
            "an explicit package-local declaration target is required",
        ));
    }
    Ok(target)
}

fn select(
    value: &OrderedValue,
    active: &[&str],
    entered_types: bool,
) -> Result<Option<String>, Refusal> {
    match value {
        OrderedValue::Text(target) if entered_types => Ok(Some(target.clone())),
        OrderedValue::Text(_) => Err(Refusal::new(
            "condition-before-types",
            "an earlier matching condition wins before the explicit types branch",
        )),
        OrderedValue::Array => Err(Refusal::new(
            "unsupported-export-array",
            "array export fallback is not admitted by this bounded resolver",
        )),
        OrderedValue::Other => Err(Refusal::new(
            "blocked-export-target",
            "the selected export branch does not expose a declaration target",
        )),
        OrderedValue::Object(entries) => {
            for (condition, target) in entries {
                if condition.starts_with('.') {
                    return Err(Refusal::new(
                        "nested-subpath",
                        "nested subpath exports are ambiguous",
                    ));
                }
                if condition.starts_with("types@") {
                    return Err(Refusal::new(
                        "versioned-type-condition",
                        "versioned type conditions require the native semantic resolver",
                    ));
                }
                if (condition == "default" || active.contains(&condition.as_str()))
                    && let Some(target) =
                        select(target, active, entered_types || condition == "types")?
                {
                    return Ok(Some(target));
                }
            }
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::experimental_type_target;

    const PREVIEW: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/typechecker/experimental-router-project-capability/preview/package.json.txt"
    ));
    const PUBLISHED: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/typechecker/experimental-router-project-capability/published/package.json.txt"
    ));

    #[test]
    fn both_real_same_version_manifests_have_one_explicit_type_target() {
        for manifest in [PUBLISHED, PREVIEW] {
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(manifest).unwrap()["version"],
                "5.4.0"
            );
            for conditions in [
                &["types", "import"][..],
                &["types", "require"],
                &["types", "node", "import"],
                &["types", "node", "require"],
            ] {
                assert_eq!(
                    experimental_type_target(manifest, conditions).unwrap(),
                    "./dist/experimental/index.d.ts"
                );
            }
        }
    }

    #[test]
    fn raw_default_and_custom_precedence_never_become_sorted_types_precedence() {
        let default_first = r#"{"exports":{"./experimental":{"default":"./dist/vue-router.d.ts","types":"./dist/experimental/index.d.ts"}}}"#;
        assert_eq!(
            experimental_type_target(default_first, &["types", "import"])
                .unwrap_err()
                .code,
            "condition-before-types"
        );
        let custom_first = r#"{"exports":{"./experimental":{"production":"./dist/vue-router.d.ts","types":"./dist/experimental/index.d.ts"}}}"#;
        assert_eq!(
            experimental_type_target(custom_first, &["types", "import", "production"])
                .unwrap_err()
                .code,
            "condition-before-types"
        );
        assert_eq!(
            experimental_type_target(custom_first, &["types", "import"]).unwrap(),
            "./dist/experimental/index.d.ts"
        );
        let types_first = r#"{"exports":{"./experimental":{"types":"./dist/experimental/index.d.ts","default":"./dist/vue-router.d.ts"}}}"#;
        assert_eq!(
            experimental_type_target(types_first, &["types", "import"]).unwrap(),
            "./dist/experimental/index.d.ts"
        );
    }

    #[test]
    fn duplicate_ambiguous_and_incomplete_targets_are_explicit_refusals() {
        for (source, code) in [
            (
                r#"{"exports":{"./experimental":{"types":"./a.d.ts"},"default":"./a.d.ts"}}"#,
                "mixed-exports-root",
            ),
            (
                r#"{"exports":{"./experimental":{"types":"./a.d.ts","types":"./b.d.ts"}}}"#,
                "invalid-package-json",
            ),
            (
                r#"{"exports":{"./experimental":{"types":["./a.d.ts","./b.d.ts"]}}}"#,
                "unsupported-export-array",
            ),
            (
                r#"{"exports":{"./experimental":{"types":"../outside.d.ts"}}}"#,
                "unsupported-type-target",
            ),
            (
                r#"{"exports":{"./experimental":{"types":"./dist%2fexperimental/index.d.ts"}}}"#,
                "unsupported-type-target",
            ),
            (
                r#"{"exports":{"./experimental":{"types":"./dist/node_modules/index.d.ts"}}}"#,
                "unsupported-type-target",
            ),
            (
                r#"{"exports":{"./experimental":{"types":null}}}"#,
                "blocked-export-target",
            ),
            (
                r#"{"exports":{"./experimental":{"types@>=5.0":"./a.d.ts","types":"./b.d.ts"}}}"#,
                "versioned-type-condition",
            ),
        ] {
            assert_eq!(
                experimental_type_target(source, &["types", "import"])
                    .unwrap_err()
                    .code,
                code
            );
        }
    }
}
