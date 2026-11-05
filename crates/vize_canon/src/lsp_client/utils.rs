use super::LspDiagnostic;
use lsp_types::Diagnostic;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use vize_l0::{FxHashMap, String, cstr};

pub(super) fn value_to_json<T>(value: T) -> Result<Value, String>
where
    T: Serialize,
{
    serde_json::to_value(value).map_err(|e| cstr!("Failed to encode JSON response: {e}"))
}

pub(super) fn convert_diagnostics<T: std::borrow::Borrow<Diagnostic>>(
    diagnostics: &[T],
) -> Vec<LspDiagnostic> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| {
            serde_json::to_value(diagnostic.borrow())
                .ok()
                .and_then(|value| serde_json::from_value(value).ok())
        })
        .collect()
}

pub(super) fn remap_json_uris(value: &mut Value, mappings: &FxHashMap<String, String>) {
    remap_protocol_uris(value, &|uri| {
        mappings
            .get(crate::file_uri::file_uri_identity(uri).as_str())
            .cloned()
    });
}

pub(super) fn remap_query_uris(
    value: &mut Value,
    mappings: &FxHashMap<String, String>,
    request: &str,
    authored_root: &std::path::Path,
) {
    let mut spellings = Vec::new();
    if let Some(path) = crate::file_uri::file_uri_to_path(request) {
        for authored in path
            .parent()
            .into_iter()
            .flat_map(std::path::Path::ancestors)
        {
            let Ok(physical) = authored.canonicalize() else {
                break;
            };
            let physical =
                crate::file_uri::file_uri_to_path(&crate::file_uri::path_to_file_uri(&physical));
            let Some(physical) = physical else {
                break;
            };
            if !super::session_paths::is_within_project_namespace(&physical, authored_root) {
                break;
            }
            if physical == authored {
                break;
            }
            spellings.push((physical, authored.to_path_buf()));
        }
    }
    remap_protocol_uris(value, &|uri| {
        let identity = crate::file_uri::file_uri_identity(uri);
        let mapped = mappings.get(identity.as_str());
        let external = mapped.map_or(identity.as_str(), String::as_str);
        if let Some(path) = crate::file_uri::file_uri_to_path(external) {
            for (physical, authored) in &spellings {
                if let Ok(relative) = path.strip_prefix(physical) {
                    return Some(crate::file_uri::path_to_file_uri(&authored.join(relative)));
                }
            }
            // Closed dependencies no longer have an open-document spelling.
            // Keep their owned URI identity consistent with open targets, while
            // preserving explicit caller mappings and foreign native results.
            if mapped.is_none()
                && identity != uri
                && super::session_paths::is_within_project_namespace(&path, authored_root)
            {
                return Some(identity);
            }
        }
        mapped.cloned()
    });
}

pub(super) fn remap_protocol_uris(value: &mut Value, mapping: &impl Fn(&str) -> Option<String>) {
    match value {
        Value::Array(items) => {
            for item in items {
                remap_protocol_uris(item, mapping);
            }
        }
        Value::Object(object) => {
            for (key, item) in object.iter_mut() {
                match key.as_str() {
                    "data" | "command" | "arguments" => {}
                    "uri" | "targetUri" | "oldUri" | "newUri" => {
                        if let Some(mapped) = item.as_str().and_then(mapping) {
                            *item = Value::String(mapped.as_str().into());
                        }
                    }
                    "changes" => {
                        if let Some(changes) = item.as_object_mut() {
                            for (uri, edits) in std::mem::take(changes) {
                                let target = mapping(&uri)
                                    .map(|mapped| mapped.as_str().into())
                                    .unwrap_or(uri);
                                match changes.entry(target) {
                                    serde_json::map::Entry::Vacant(entry) => {
                                        entry.insert(edits);
                                    }
                                    serde_json::map::Entry::Occupied(mut entry) => {
                                        match (entry.get_mut(), edits) {
                                            (Value::Array(existing), Value::Array(incoming)) => {
                                                existing.extend(incoming);
                                            }
                                            (existing, incoming) => *existing = incoming,
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => remap_protocol_uris(item, mapping),
                }
            }
        }
        _ => {}
    }
}

pub(super) fn remap_serialized_uris<T>(
    value: T,
    mappings: &FxHashMap<String, String>,
) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
{
    let mut value = serde_json::to_value(value)
        .map_err(|error| cstr!("Failed to encode Corsa result: {error}"))?;
    remap_json_uris(&mut value, mappings);
    serde_json::from_value(value).map_err(|error| cstr!("Failed to decode Corsa result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::remap_json_uris;
    use serde_json::json;
    use vize_l0::{FxHashMap, String};

    #[test]
    fn converging_workspace_edit_keys_keep_every_edit_in_each_original_order() {
        let authored = "file:///authored/source.ts";
        let first = "file:///native/first.ts";
        let second = "file:///native/second.ts";
        let mappings = FxHashMap::from_iter([
            (String::from(first), String::from(authored)),
            (String::from(second), String::from(authored)),
        ]);
        let mut response = json!({"changes":{
            authored:[{"newText":"authored"}],
            first:[{"newText":first},{"newText":"first-tail"}],
            second:[{"newText":second}],
        },"data":{"uri":first}});
        remap_json_uris(&mut response, &mappings);
        assert_eq!(
            response,
            json!({"changes":{authored:[
            {"newText":"authored"},{"newText":first},{"newText":"first-tail"},{"newText":second}
        ]},"data":{"uri":first}})
        );
    }

    #[test]
    fn closed_owned_targets_keep_identity_after_their_open_mapping_is_removed() {
        use crate::file_uri::path_to_file_uri;
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let workspace = root.join("workspace");
        std::fs::create_dir_all(workspace.join("src")).unwrap();
        let request = path_to_file_uri(&workspace.join("src/App.vue.ts"));
        let caller = path_to_file_uri(&workspace.join("src/format.ts"));
        let encoded = caller.replace("format.ts", "format%2Ets");
        let foreign =
            path_to_file_uri(&root.join("foreign.ts")).replace("foreign.ts", "foreign%2Ets");
        let known = "file:///native/known.ts";
        let mappings = FxHashMap::from_iter([(String::from(known), encoded.clone().into())]);
        let mut response = json!({
            "uri":encoded,"targetUri":known,
            "locations":[{"uri":foreign}],"data":{"uri":encoded},
            "edit":{"changes":{encoded.clone():[{"newText":encoded}]}},
        });
        super::remap_query_uris(&mut response, &mappings, &request, &workspace);
        assert_eq!(
            response,
            json!({
                "uri":caller,"targetUri":encoded,
                "locations":[{"uri":foreign}],"data":{"uri":encoded},
                "edit":{"changes":{caller:[{"newText":encoded}]}},
            })
        );
    }

    #[cfg(unix)]
    #[test]
    fn query_uris_keep_the_request_spelling_after_native_remapping() {
        use crate::file_uri::path_to_file_uri;
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let workspace = root.join("workspace");
        let linked = root.join("linked");
        std::fs::create_dir_all(workspace.join("src")).unwrap();
        std::os::unix::fs::symlink(&workspace, &linked).unwrap();
        let request = path_to_file_uri(&linked.join("src/App.vue.ts"));
        let physical = path_to_file_uri(&workspace.join("src/format.ts"));
        let authored = path_to_file_uri(&linked.join("src/format.ts"));
        let foreign = path_to_file_uri(&root.join("foreign/format.ts"));
        let native = "file:///native/format.ts";
        let mappings = FxHashMap::from_iter([(String::from(native), physical.clone())]);
        let mut response = json!({
            "uri":physical, "targetUri":native,
            "locations":[{"uri":foreign}],
            "edit":{"changes":{native:[{"newText":native}]}},
            "data":{"uri":physical}, "command":{"arguments":[{"uri":native}]},
        });
        super::remap_query_uris(&mut response, &mappings, &request, &workspace);
        assert_eq!(
            response,
            json!({
                "uri":authored, "targetUri":authored,
                "locations":[{"uri":foreign}],
                "edit":{"changes":{authored:[{"newText":native}]}},
                "data":{"uri":physical}, "command":{"arguments":[{"uri":native}]},
            })
        );
    }

    #[test]
    fn remapping_changes_only_protocol_uris_and_workspace_edit_keys() {
        let native = "file:///native/part:name.ts";
        let external = "file:///authored/part:name.ts";
        let mappings = FxHashMap::from_iter([(String::from(native), String::from(external))]);
        let mut response = json!({
            "uri":native, "targetUri":"file:///native/part%3Aname.ts",
            "oldUri":native, "newUri":native,
            "edit":{"changes":{native:[{"newText":native}]}},
            "data":{"uri":native}, "command":{"arguments":[{"uri":native}]},
            "message":native, "contents":native,
        });
        remap_json_uris(&mut response, &mappings);
        assert_eq!(
            response,
            json!({
                "uri":external, "targetUri":external,
                "oldUri":external, "newUri":external,
                "edit":{"changes":{external:[{"newText":native}]}},
                "data":{"uri":native}, "command":{"arguments":[{"uri":native}]},
                "message":native, "contents":native,
            })
        );
    }
}
