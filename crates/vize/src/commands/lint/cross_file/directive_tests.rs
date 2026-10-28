//! Whole published original packets; no diagnostic field is projected away.

use super::build_cross_file_lint_output;
use std::path::PathBuf;
use vize_l0::String;
use vize_patina::{HelpLevel, OutputFormat, format_results};

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/cross-file-directives-10912"
);

fn files() -> Vec<(PathBuf, std::string::String)> {
    ["Editor.vue", "PageA.vue", "PageB.vue"]
        .into_iter()
        .map(|name| {
            let file = std::path::Path::new(FIXTURES)
                .join("original/src")
                .join(format!("{name}.txt"));
            (
                PathBuf::from(format!("src/{name}")),
                std::fs::read_to_string(file).unwrap(),
            )
        })
        .collect()
}

fn packet(files: &[(PathBuf, std::string::String)]) -> serde_json::Value {
    let output = build_cross_file_lint_output(files, HelpLevel::Full, false);
    let sources: Vec<(String, String)> = files
        .iter()
        .map(|(path, source)| {
            (
                path.to_string_lossy().as_ref().into(),
                source.as_str().into(),
            )
        })
        .collect();
    serde_json::from_str(format_results(&output.results, &sources, OutputFormat::Json).as_str())
        .unwrap()
}

fn golden(name: &str) -> serde_json::Value {
    serde_json::from_str(
        &std::fs::read_to_string(std::path::Path::new(FIXTURES).join(name)).unwrap(),
    )
    .unwrap()
}

#[test]
fn original_canonical_eslint_and_oxlint_comments_suppress_only_their_findings() {
    let expected = golden("original-after.packet.json");
    for prefix in ["eslint", "oxlint"] {
        let files: Vec<_> = files()
            .into_iter()
            .map(|(path, source)| {
                (
                    path,
                    source.replace("eslint-disable", &format!("{prefix}-disable")),
                )
            })
            .collect();
        for _ in 0..2 {
            assert_eq!(packet(&files), expected, "{prefix}");
        }
    }
}

#[test]
fn group_printed_bare_and_unrelated_names_do_not_mask_the_original_findings() {
    let expected = golden("original-before.packet.json");
    for replacement in [
        "cross-file",
        "vize:croquis/cf/spread-breaks-reactivity",
        "spread-breaks-reactivity",
        "croquis/cf/unused-provide",
    ] {
        let files: Vec<_> = files()
            .into_iter()
            .map(|(path, source)| {
                (
                    path,
                    source
                        .replace("croquis/cf/spread-breaks-reactivity", replacement)
                        .replace("croquis/cf/duplicate-id", replacement),
                )
            })
            .collect();
        for _ in 0..2 {
            assert_eq!(packet(&files), expected, "{replacement}");
        }
    }
}

#[test]
fn unsupported_vize_alias_stays_a_negative_control_for_the_existing_grammar() {
    let files: Vec<_> = files()
        .into_iter()
        .map(|(path, source)| (path, source.replace("eslint-disable", "vize-disable")))
        .collect();
    assert_eq!(packet(&files), golden("original-before.packet.json"));
}

#[test]
fn one_matching_script_comment_cannot_claim_the_other_file_diagnostic() {
    let files: Vec<_> = files()
        .into_iter()
        .map(|(path, source)| {
            (
                path,
                source.replace("croquis/cf/duplicate-id", "croquis/cf/unused-provide"),
            )
        })
        .collect();
    let mut expected = golden("original-before.packet.json");
    expected[0]["messages"] = serde_json::json!([]);
    expected[0]["errorCount"] = serde_json::json!(0);
    assert_eq!(packet(&files), expected);
}
