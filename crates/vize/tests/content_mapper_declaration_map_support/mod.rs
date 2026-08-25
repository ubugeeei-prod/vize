use std::path::{Path, PathBuf};

use serde_json::Value;

pub struct AuthoredVueSource {
    pub map_source: String,
    pub content: String,
}

pub fn assert_authored_vue_declaration_map(
    project_root: &Path,
    declaration: &str,
    component: &str,
) -> AuthoredVueSource {
    let declaration_path = project_root.join(declaration);
    let declaration_text = std::fs::read_to_string(&declaration_path).unwrap();
    let name = declaration_path.file_name().unwrap().to_string_lossy();
    let expected_mapping_url = format!("//# sourceMappingURL={name}.map");
    assert_eq!(
        declaration_text.lines().last(),
        Some(expected_mapping_url.as_str()),
        "declaration must end with an adjacent sourceMappingURL:\n{declaration_text}"
    );

    let map_path = declaration_path.with_file_name(format!("{name}.map"));
    let map_text = std::fs::read_to_string(&map_path).unwrap();
    let map: Value = serde_json::from_str(&map_text).unwrap();
    assert_eq!(map["file"], name.as_ref());
    let expected_source = format!("../src/{component}.vue");
    assert_eq!(map["sources"], serde_json::json!([expected_source]));

    let authored_source = assert_authored_vue_source(&map_path, &map, component);
    assert_eq!(authored_source.map_source, expected_source);
    authored_source
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
