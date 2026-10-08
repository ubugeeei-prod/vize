use std::{io, path::Path};

use serde_json::Value;

use super::super::chain::ConfigChain;
use super::super::jsonc::parse_jsonc_value;
use super::super::loader::{
    read_extends_entries, resolve_extended_tsconfig, tracked_read_to_string,
};

pub(crate) fn collect_tsconfig_type_packages(
    tsconfig_path: Option<&Path>,
) -> Vec<std::string::String> {
    let Some(tsconfig_path) = tsconfig_path else {
        return Vec::new();
    };

    load_tsconfig_type_packages(tsconfig_path, &mut ConfigChain::default())
        .ok()
        .flatten()
        .unwrap_or_default()
}

fn load_tsconfig_type_packages(
    tsconfig_path: &Path,
    chain: &mut ConfigChain<Option<Vec<std::string::String>>>,
) -> io::Result<Option<Vec<std::string::String>>> {
    let resolved = vize_carton::path::canonicalize_non_verbatim(tsconfig_path);
    chain.load(&resolved, |chain| {
        let content = tracked_read_to_string(&resolved)?;
        let value = parse_jsonc_value(&content).map_err(io::Error::other)?;

        let mut inherited = None;
        for extends in read_extends_entries(&value) {
            let Some(extends_path) = resolve_extended_tsconfig(&resolved, &extends) else {
                continue;
            };
            // A failed parent keeps the preceding valid sibling, as before. An
            // explicitly empty array is present and replaces that sibling too.
            if let Ok(Some(parent_types)) = load_tsconfig_type_packages(&extends_path, chain) {
                inherited = Some(parent_types);
            }
        }

        if let Some(types) = value
            .get("compilerOptions")
            .and_then(Value::as_object)
            .and_then(|compiler_options| compiler_options.get("types"))
            .and_then(Value::as_array)
        {
            return Ok(Some(
                types
                    .iter()
                    .filter_map(Value::as_str)
                    .map(std::string::String::from)
                    .collect(),
            ));
        }

        Ok(inherited)
    })
}

#[cfg(test)]
#[path = "tsconfig_types/extends_tests.rs"]
mod extends_tests;
