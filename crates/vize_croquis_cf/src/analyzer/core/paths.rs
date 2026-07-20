use std::path::{Path, PathBuf};
use vize_carton::String;

pub(super) fn import_candidates(specifier: &str, from_dir: Option<&Path>) -> Vec<PathBuf> {
    crate::module_paths::import_candidates(specifier, from_dir)
}

pub(super) fn component_names_match(left: &str, right: &str) -> bool {
    left == right || to_pascal_case(left) == to_pascal_case(right)
}

fn to_pascal_case(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::default(),
            }
        })
        .collect()
}
