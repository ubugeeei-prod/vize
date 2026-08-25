use std::path::{Path, PathBuf};

use serde_json::Value;

pub struct AuthoredVueSource {
    pub map_source: String,
    pub content: String,
}

pub fn assert_authored_vue_source(
    map_path: &Path,
    map: &Value,
    component: &str,
) -> AuthoredVueSource {
    let sources = map["sources"].as_array().expect("map sources");
    let sources_content = map["sourcesContent"].as_array();
    if let Some(sources_content) = sources_content {
        assert_eq!(
            sources_content.len(),
            sources.len(),
            "{} must keep source and sourcesContent entries aligned: {map}",
            map_path.display()
        );
    }

    let expected_source = format!("{component}.vue");
    let mut matched_content = None;
    for (index, source_value) in sources.iter().enumerate() {
        let source = source_value
            .as_str()
            .unwrap_or_else(|| panic!("{} has non-string source entry: {map}", map_path.display()));
        if !source.ends_with(&expected_source) {
            continue;
        }
        let authored = std::fs::read_to_string(resolve_map_source(map_path, source))
            .unwrap_or_else(|error| {
                panic!(
                    "failed to read authored source {source} for {}: {error}",
                    map_path.display()
                )
            });
        if let Some(sources_content) = sources_content {
            let actual = sources_content[index].as_str().unwrap_or_else(|| {
                panic!(
                    "{} has non-string sourcesContent for {source}: {map}",
                    map_path.display()
                )
            });
            assert_eq!(
                actual,
                authored,
                "{} must embed byte-exact authored source content for {source}",
                map_path.display()
            );
        }
        matched_content = Some(AuthoredVueSource {
            map_source: source.to_string(),
            content: authored,
        });
    }

    matched_content.unwrap_or_else(|| {
        panic!(
            "{} did not resolve an authored source for {expected_source}: {map}",
            map_path.display()
        )
    })
}

fn resolve_map_source(map_path: &Path, source: &str) -> PathBuf {
    let source_path = Path::new(source);
    if source_path.is_absolute() {
        source_path.to_path_buf()
    } else {
        map_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(source_path)
    }
}
