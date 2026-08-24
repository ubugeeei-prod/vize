use std::path::{Path, PathBuf};

use serde::Deserialize;
use vize_carton::String;

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
    assert!(!matrix.unproven.is_empty());
    for required in &matrix.required_coverage {
        assert!(
            matrix
                .fixtures
                .iter()
                .any(|fixture| fixture.coverage.contains(required)),
            "missing required TS-40 coverage: {required}"
        );
    }
    matrix
}

pub(super) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
