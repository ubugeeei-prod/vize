use super::*;

/// Rule ids and catalogue links from `docs/content/rules/all.md`.
fn published_rule_pages(catalogue: &str) -> Vec<(&str, &str)> {
    let mut pages = Vec::new();
    let mut rest = catalogue;
    while let Some(start) = rest.find("[`") {
        rest = &rest[start + 2..];
        let Some(id_end) = rest.find("`](./") else {
            break;
        };
        let id = &rest[..id_end];
        rest = &rest[id_end + 5..];
        let Some(path_end) = rest.find(')') else {
            break;
        };
        let relative = &rest[..path_end];
        rest = &rest[path_end + 1..];
        if relative.starts_with("reference/") || relative.starts_with("project/") {
            pages.push((id, relative));
        }
    }
    pages
}

#[test]
fn rule_docs_path_exists_for_every_published_rule() {
    const CATALOGUE: &str = include_str!("../../../../docs/content/rules/all.md");
    let pages = published_rule_pages(CATALOGUE);
    let reference = pages
        .iter()
        .filter(|(_, relative)| relative.starts_with("reference/"))
        .count();
    let project = pages.len() - reference;
    assert!(
        reference >= 200 && project >= 60,
        "published catalogue was not read: reference={reference} project={project}"
    );

    let rules_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/content/rules");
    let mut missing = vize_l0::String::new("");
    for (id, relative) in pages {
        let path = rule_docs_path(id);
        // This operational check retains its established guide authority. The
        // generated rule page supplies examples and links back to that guide.
        let expected = if id == "vue/comment-directive" {
            vize_l0::String::from("docs/content/guide/lint-execution.md")
        } else {
            vize_l0::cstr!("docs/content/rules/{relative}")
        };
        assert_eq!(path, expected, "{id}");
        if !rules_dir.join(relative).is_file() {
            if !missing.is_empty() {
                missing.push('\n');
            }
            vize_l0::append!(missing, "{id} -> {path}");
        }
    }
    assert!(
        missing.is_empty(),
        "published rules with no page:\n{missing}"
    );
}

#[test]
fn synthetic_diagnostic_routes_keep_their_owned_documentation() {
    const ROUTES: &str = include_str!(
        "../../../../tests/_fixtures/differential/linter/diagnostic-documentation-11818/routes.json"
    );
    const CONTROL: &str = include_str!(
        "../../../../tests/_fixtures/differential/linter/diagnostic-documentation-11818/DocumentationControl.vue.txt"
    );
    let input: serde_json::Value = serde_json::from_str(ROUTES).unwrap();
    assert_eq!(
        input["schema"],
        "vize.synthetic-diagnostic-documentation.v1"
    );
    assert_eq!(input["version"], 1);
    let routes = input["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 2);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for route in routes {
        let id = route["id"].as_str().unwrap();
        let page = route["page"].as_str().unwrap();
        let expected = vize_l0::cstr!("docs/content/rules/{page}");
        assert_eq!(rule_docs_path(id), expected, "{id}");
        for locale in ["", "ja/"] {
            let path = vize_l0::cstr!("docs/content/{locale}rules/{page}");
            assert!(root.join(path.as_str()).is_file(), "{id}: {path}");
        }
    }
    assert_eq!(
        rule_docs_path(input["unknown"]["id"].as_str().unwrap()).as_str(),
        input["unknown"]["path"].as_str().unwrap()
    );
    let filename = vize_l0::String::from("DocumentationControl.vue");
    let actual = format_results(
        &[Linter::new().lint_sfc(CONTROL, &filename)],
        &[(filename, CONTROL.into())],
        OutputFormat::Json,
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&actual).unwrap(),
        serde_json::json!([{"file":"DocumentationControl.vue","messages":[],"errorCount":0,"warningCount":0}])
    );
}
