//! Conditional package exports for the owning TypeScript project's conditions.
//!
//! Preserve manifest key order locally: serde_json's default object sorts keys,
//! but conditional export priority is the package author's declaration order.

use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, SeqAccess, Visitor},
};
use std::{
    fmt,
    path::{Path, PathBuf},
};
use vize_carton::{String, ToCompactString, cstr};

#[derive(Deserialize)]
struct Manifest {
    exports: Target,
}

enum Target {
    Path(String),
    Object(Vec<(String, Target)>),
    Array(Vec<Target>),
    Null,
}

impl<'de> Deserialize<'de> for Target {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OrderedTarget;
        impl<'de> Visitor<'de> for OrderedTarget {
            type Value = Target;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a package export target")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Target, E> {
                Ok(Target::Path(value.to_compact_string()))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Target, E> {
                Ok(Target::Null)
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Target, M::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    entries.push(entry);
                }
                Ok(Target::Object(entries))
            }
            fn visit_seq<S: SeqAccess<'de>>(self, mut sequence: S) -> Result<Target, S::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = sequence.next_element()? {
                    entries.push(entry);
                }
                Ok(Target::Array(entries))
            }
        }
        deserializer.deserialize_any(OrderedTarget)
    }
}

pub(super) enum Selection {
    Path(PathBuf),
    Excluded,
}

pub(super) fn resolve(
    package: &Path,
    manifest: &str,
    subpath: &str,
    conditions: &[String],
    probe: impl Fn(PathBuf) -> Option<PathBuf>,
) -> Option<Selection> {
    let manifest: Manifest = serde_json::from_str(manifest).ok()?;
    let key = if subpath.is_empty() {
        String::from(".")
    } else {
        cstr!("./{subpath}")
    };
    let mut wildcard = None;
    let target = match &manifest.exports {
        Target::Object(entries) if entries.iter().any(|(key, _)| key.starts_with('.')) => {
            if let Some((_, target)) = entries.iter().find(|(entry, _)| entry == key) {
                target
            } else {
                let mut matched = None;
                for (pattern, target) in entries {
                    let Some((prefix, suffix)) = pattern.split_once('*') else {
                        continue;
                    };
                    let Some(captured) = key
                        .strip_prefix(prefix)
                        .and_then(|rest| rest.strip_suffix(suffix))
                    else {
                        continue;
                    };
                    if matched.is_none_or(|(before_prefix, before_suffix, _, _)| {
                        (prefix.len(), suffix.len()) > (before_prefix, before_suffix)
                    }) {
                        matched = Some((prefix.len(), suffix.len(), captured, target));
                    }
                }
                let Some((_, _, captured, target)) = matched else {
                    return Some(Selection::Excluded);
                };
                wildcard = Some(captured);
                target
            }
        }
        target if subpath.is_empty() => target,
        _ => return Some(Selection::Excluded),
    };
    // Once an exports map owns the route, an inactive or missing target
    // must not fall back to unrelated top-level/physical declarations.
    select(target, package, wildcard, conditions, &probe).or(Some(Selection::Excluded))
}

fn select(
    target: &Target,
    package: &Path,
    wildcard: Option<&str>,
    conditions: &[String],
    probe: &impl Fn(PathBuf) -> Option<PathBuf>,
) -> Option<Selection> {
    match target {
        Target::Path(target) => {
            let relative = target.strip_prefix("./")?;
            let relative = wildcard.map_or_else(
                || relative.to_compact_string(),
                |capture| relative.replace('*', capture).to_compact_string(),
            );
            if relative
                .split('/')
                .any(|segment| segment == ".." || segment == "node_modules")
            {
                return None;
            }
            probe(package.join(relative.as_str())).map(Selection::Path)
        }
        Target::Null => Some(Selection::Excluded),
        Target::Object(entries) => entries
            .iter()
            .filter(|(condition, _)| {
                condition == "default" || conditions.iter().any(|active| active == condition)
            })
            .find_map(|(_, target)| select(target, package, wildcard, conditions, probe)),
        // TypeScript stops on an explicit null array entry; absent file
        // targets may continue to a later array member.
        Target::Array(targets) => targets
            .iter()
            .find_map(|target| select(target, package, wildcard, conditions, probe)),
    }
}
