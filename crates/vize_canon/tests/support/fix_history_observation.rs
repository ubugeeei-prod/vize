//! Receipted observations of the existing required batch-fixture executions.
//! The writer runs only when a capture directory is explicitly supplied.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(clippy::disallowed_types, reason = "JSON fixtures use std strings")]

use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use vize_canon::BatchTypeCheckResult;

use super::super::{Diagnostic, Input};

#[derive(Serialize)]
struct ObservedInput {
    file: String,
    sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ObservedCase {
    id: String,
    project_root: PathBuf,
    inputs: Vec<ObservedInput>,
    diagnostics: Vec<Diagnostic>,
    public_result: PublicResult,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicDiagnostic {
    file: PathBuf,
    line: u32,
    column: u32,
    severity: u8,
    code: Option<u32>,
    message: String,
    block_type: Option<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicResult {
    exit_code: i32,
    success: bool,
    diagnostics: Vec<PublicDiagnostic>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Observation<'a> {
    schema: &'static str,
    version: u32,
    pack: &'a str,
    test: &'a str,
    fixture_pack_sha256: String,
    archive_receipt_sha256: String,
    binary_path: &'a Path,
    binary_sha256: String,
    missing_fields: [&'static str; 3],
    matched_contract: &'static str,
    unbaselined_fields: [&'static str; 3],
    cases: &'a [ObservedCase],
}

pub(crate) struct Capture {
    directory: PathBuf,
    archive_receipt_sha256: String,
    binary_path: PathBuf,
    binary_sha256: String,
    cases: Vec<ObservedCase>,
}

#[expect(
    clippy::disallowed_macros,
    reason = "the JSON observation requires a std String hexadecimal digest"
)]
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl Capture {
    pub(crate) fn from_environment() -> Option<Self> {
        let directory = std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR")?;
        let receipt = std::env::var_os("VIZE_TEST_FIX_HISTORY_ARCHIVE_RECEIPT")
            .expect("receipted capture requires the verified Cargo archive receipt");
        let executable = std::env::current_exe().expect("fixture executable identity must resolve");
        Some(Self {
            directory: PathBuf::from(directory),
            archive_receipt_sha256: digest(
                &std::fs::read(receipt).expect("verified Cargo archive receipt must exist"),
            ),
            binary_sha256: digest(
                &std::fs::read(&executable).expect("actual fixture executable must be readable"),
            ),
            binary_path: executable,
            cases: Vec::new(),
        })
    }

    // Only called after the unchanged complete diagnostic assertion succeeds.
    pub(crate) fn record(
        &mut self,
        id: String,
        inputs: &[Input],
        project: &Path,
        diagnostics: Vec<Diagnostic>,
        result: &BatchTypeCheckResult,
    ) {
        let inputs = inputs
            .iter()
            .map(|input| ObservedInput {
                file: input.file.clone(),
                sha256: digest(
                    &std::fs::read(project.join(&input.file))
                        .expect("actual materialized input bytes must remain available"),
                ),
            })
            .collect();
        self.cases.push(ObservedCase {
            id,
            project_root: project.to_path_buf(),
            inputs,
            diagnostics,
            public_result: PublicResult {
                exit_code: result.exit_code,
                success: result.success,
                diagnostics: result
                    .diagnostics
                    .iter()
                    .map(|diagnostic| PublicDiagnostic {
                        file: diagnostic.file.clone(),
                        line: diagnostic.line,
                        column: diagnostic.column,
                        severity: diagnostic.severity,
                        code: diagnostic.code,
                        message: String::from(diagnostic.message.as_str()),
                        block_type: diagnostic.block_type.map(|block| block.block_name()),
                    })
                    .collect(),
            },
        });
    }

    pub(crate) fn finish(self, pack: &str, test: &str, fixture_pack: &[u8]) {
        let observation = Observation {
            schema: "vize.typechecker-fixture-observation",
            version: 1,
            pack,
            test,
            fixture_pack_sha256: digest(fixture_pack),
            archive_receipt_sha256: self.archive_receipt_sha256,
            binary_path: &self.binary_path,
            binary_sha256: self.binary_sha256,
            missing_fields: ["end", "relatedInformation", "raw backend diagnostics"],
            matched_contract: "batch-start-diagnostics-v1",
            unbaselined_fields: ["diagnostics[].blockType", "exitCode", "success"],
            cases: &self.cases,
        };
        std::fs::create_dir_all(&self.directory).expect("capture directory must be created");
        let destination = self.directory.join(pack).with_extension("json");
        let mut temporary = tempfile::NamedTempFile::new_in(&self.directory)
            .expect("atomic capture file must be created");
        serde_json::to_writer_pretty(temporary.as_file_mut(), &observation)
            .expect("actual fixture observations must serialize");
        temporary
            .as_file()
            .sync_all()
            .expect("complete observation must flush before publication");
        temporary
            .persist_noclobber(destination)
            .expect("one complete capture per test body must publish atomically");
    }
}
