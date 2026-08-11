//! Manifest target topology collected without selecting a condition branch.

use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use vize_carton::String;

pub(super) fn collect_types_version_candidates(
    manifest: &Value,
    request: &str,
    root: &Path,
    candidates: &mut Vec<PathBuf>,
) {
    let Some(versions) = manifest.get("typesVersions").and_then(Value::as_object) else {
        return;
    };
    let requests = if request == "." {
        let mut requests = ["types", "typings", "main"]
            .into_iter()
            .filter_map(|field| manifest.get(field).and_then(Value::as_str))
            .map(|target| target.trim_start_matches("./").to_owned())
            .collect::<Vec<_>>();
        requests.push("index".to_owned());
        requests
    } else {
        vec![request.trim_start_matches("./").to_owned()]
    };
    for mappings in versions.values().filter_map(Value::as_object) {
        for (pattern, targets) in mappings {
            for request in &requests {
                let capture = if let Some((prefix, suffix)) = pattern.split_once('*') {
                    request
                        .strip_prefix(prefix)
                        .and_then(|value| value.strip_suffix(suffix))
                } else {
                    (pattern == request).then_some("")
                };
                let Some(capture) = capture else {
                    continue;
                };
                for target in targets
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    push_relative_target(&target.replace('*', capture), root, candidates);
                }
            }
        }
    }
}

pub(super) fn collect_legacy_candidates(
    manifest: &Value,
    request: &str,
    root: &Path,
    candidates: &mut Vec<PathBuf>,
) {
    if request != "." {
        candidates.push(root.join(request.trim_start_matches("./")));
        return;
    }
    for field in ["types", "typings", "module", "main"] {
        if let Some(target) = manifest.get(field).and_then(Value::as_str) {
            candidates.push(root.join(target.trim_start_matches("./")));
        }
    }
    candidates.push(root.join("index"));
}

pub(super) fn collect_request_targets(
    value: &Value,
    request: &str,
    root: &Path,
    out: &mut Vec<PathBuf>,
) {
    if let Some(mappings) = value.as_object()
        && mappings.keys().any(|key| key.starts_with(['.', '#']))
    {
        if let Some(target) = mappings.get(request) {
            collect_targets(target, root, None, out);
            return;
        }
        let matches = mappings.iter().filter_map(|(pattern, value)| {
            let (prefix, suffix) = pattern.split_once('*')?;
            let capture = request.strip_prefix(prefix)?.strip_suffix(suffix)?;
            Some((capture, value))
        });
        // Materialize every matching pattern candidate. The raw manifest—not
        // this iteration order—lets native TypeScript apply specificity/null
        // blocking and condition rules.
        for (capture, target) in matches {
            collect_targets(target, root, Some(capture), out);
        }
        return;
    }
    if request == "." {
        collect_targets(value, root, None, out);
    }
}

pub(super) fn collect_external_import_targets(value: &Value, request: &str, out: &mut Vec<String>) {
    if let Some(mappings) = value.as_object()
        && mappings.keys().any(|key| key.starts_with(['.', '#']))
    {
        if let Some(target) = mappings.get(request) {
            collect_external_targets(target, None, out);
            return;
        }
        for (pattern, target) in mappings {
            let Some((prefix, suffix)) = pattern.split_once('*') else {
                continue;
            };
            let Some(capture) = request
                .strip_prefix(prefix)
                .and_then(|value| value.strip_suffix(suffix))
            else {
                continue;
            };
            collect_external_targets(target, Some(capture), out);
        }
        return;
    }
    if request == "." {
        collect_external_targets(value, None, out);
    }
}

fn collect_external_targets(value: &Value, wildcard: Option<&str>, out: &mut Vec<String>) {
    match value {
        Value::String(target) => {
            let target: String = wildcard.map_or_else(
                || target.as_str().into(),
                |part| target.replace('*', part).into(),
            );
            if !target.starts_with(['.', '/', '#']) && !Path::new(target.as_str()).is_absolute() {
                out.push(target);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_external_targets(value, wildcard, out);
            }
        }
        Value::Object(conditions) => {
            for value in conditions.values() {
                collect_external_targets(value, wildcard, out);
            }
        }
        _ => {}
    }
}

pub(super) fn collect_targets(
    value: &Value,
    root: &Path,
    wildcard: Option<&str>,
    out: &mut Vec<PathBuf>,
) {
    match value {
        Value::String(target) => {
            let target =
                wildcard.map_or_else(|| target.clone(), |value| target.replace('*', value));
            let Some(relative) = target.strip_prefix("./") else {
                return;
            };
            push_relative_target(relative, root, out);
        }
        Value::Array(values) => {
            for value in values {
                collect_targets(value, root, wildcard, out);
            }
        }
        Value::Object(conditions) => {
            // Preserve candidate topology, not a Vize-owned condition
            // priority. Native TypeScript interprets the unchanged manifest.
            for value in conditions.values() {
                collect_targets(value, root, wildcard, out);
            }
        }
        _ => {}
    }
}

fn push_relative_target(target: &str, root: &Path, out: &mut Vec<PathBuf>) {
    let relative = Path::new(target.trim_start_matches("./"));
    if !relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        out.push(root.join(relative));
    }
}
