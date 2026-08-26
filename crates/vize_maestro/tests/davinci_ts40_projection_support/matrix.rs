use std::path::{Path, PathBuf};

use serde::Deserialize;
use vize_s0::String;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matrix {
    pub schema_version: u8,
    pub claim: String,
    pub normalization: Vec<String>,
    pub unproven: Vec<String>,
    pub required_coverage: Vec<String>,
    pub fixtures: Vec<Fixture>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fixture {
    pub id: String,
    pub file: String,
    pub coverage: Vec<String>,
    pub line_ending: LineEnding,
    pub anchors: Vec<String>,
    pub options_api: bool,
    pub legacy_vue2: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl Fixture {
    pub fn source(&self) -> String {
        let source = std::fs::read_to_string(workspace_root().join(self.file.as_str()))
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", self.file));
        match self.line_ending {
            LineEnding::Lf => source.into(),
            LineEnding::Crlf => source.replace("\r\n", "\n").replace('\n', "\r\n").into(),
        }
    }
}

pub fn load_matrix() -> Matrix {
    let path = workspace_root().join("tests/_fixtures/davinci-ts40-projection/matrix.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let matrix: Matrix = serde_json::from_str(&text).expect("valid TS-40 matrix JSON");
    assert_eq!(matrix.schema_version, 1);
    assert_eq!(matrix.claim, "current-canon-maestro-behavior-only");
    assert!(!matrix.normalization.is_empty());
    let expected_unproven = [
        "Davinci or S2 projection parity",
        "full-corpus vize check diagnostic parity",
        "all tsgo Content Mapper editor features",
        "static slot-name navigation mappings",
        "incremental or multi-project invalidation",
        "safe deletion of either current generator",
    ];
    assert_exact_strings(&matrix.unproven, &expected_unproven);
    let expected_required_coverage = [
        "utf8",
        "crlf",
        "recovery",
        "dual-script",
        "options-api",
        "vue2",
        "generic-sfc",
        "jsx",
        "tsx",
        "props",
        "emits",
        "slots",
        "navigation-ranges",
        "local-vue-import",
    ];
    assert_exact_strings(&matrix.required_coverage, &expected_required_coverage);

    let expected_fixture_coverage: &[(&str, &[&str])] = &[
        (
            "utf8-crlf-props",
            &["utf8", "crlf", "props", "navigation-ranges"],
        ),
        ("parse-recovery", &["recovery"]),
        (
            "dual-scripts-emits",
            &["dual-script", "emits", "navigation-ranges"],
        ),
        (
            "options-api-slots",
            &["options-api", "slots", "props", "navigation-ranges"],
        ),
        (
            "generic-sfc",
            &["generic-sfc", "props", "navigation-ranges"],
        ),
        ("jsx-script", &["jsx", "navigation-ranges"]),
        ("tsx-script", &["tsx", "props", "navigation-ranges"]),
        (
            "parent-local-import",
            &["local-vue-import", "props", "navigation-ranges"],
        ),
        ("child-local-import", &["props", "navigation-ranges"]),
        ("vue2-native-event", &["vue2", "emits", "navigation-ranges"]),
    ];
    assert_eq!(matrix.fixtures.len(), expected_fixture_coverage.len());
    for (fixture, (expected_id, expected_coverage)) in
        matrix.fixtures.iter().zip(expected_fixture_coverage)
    {
        assert_eq!(fixture.id, *expected_id);
        assert_exact_strings(&fixture.coverage, expected_coverage);
    }
    matrix
}

fn assert_exact_strings(actual: &[String], expected: &[&str]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual, expected);
    }
}

pub(super) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
