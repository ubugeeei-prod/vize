//! Whole current-source CSS for the reported nested scope and its neighbors.
#![cfg(feature = "native")]

use serde::{Deserialize, Serialize};
use vize_atelier_sfc::{CssCompileOptions, compile_css};
use vize_carton::String;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pack {
    scope_id: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
}

#[derive(Serialize)]
struct Capture<'a> {
    id: &'a str,
    source: &'a str,
    minify: bool,
    code: String,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[test]
fn scope_body_source_captures_preserve_all_eleven_inputs_in_both_modes() {
    let pack: Pack =
        serde_json::from_str(include_str!("fixtures/scope-body-10810.json.txt")).unwrap();
    assert_eq!(pack.cases.len(), 11);
    let mut rows = Vec::new();
    for case in &pack.cases {
        for minify in [false, true] {
            let result = compile_css(
                &case.source,
                &CssCompileOptions {
                    filename: Some("Scope.vue".into()),
                    scoped: true,
                    scope_id: Some(pack.scope_id.clone()),
                    minify,
                    ..Default::default()
                },
            );
            assert!(result.errors.is_empty(), "{}: {:?}", case.id, result.errors);
            assert!(
                result.warnings.is_empty(),
                "{}: {:?}",
                case.id,
                result.warnings
            );
            if case.id == "reported-scope" && minify {
                assert_eq!(
                    result.code,
                    ".a[data-v-x]{@scope(.s){[data-v-x]:scope{color:#00f}.c[data-v-x]{color:red}}}"
                );
            }
            rows.push(Capture {
                id: &case.id,
                source: &case.source,
                minify,
                code: result.code,
                errors: result.errors,
                warnings: result.warnings,
            });
        }
    }
    assert_eq!(rows.len(), 22);
    if let Some(path) = std::env::var_os("VIZE_SCOPE_BODY_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}
