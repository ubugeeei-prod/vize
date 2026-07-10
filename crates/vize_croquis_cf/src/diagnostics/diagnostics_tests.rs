use super::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::FileId;
use vize_carton::append;

fn make_file_id() -> FileId {
    FileId::new(0)
}

#[path = "diagnostics_tests/codes.rs"]
mod codes;
#[path = "diagnostics_tests/markdown.rs"]
mod markdown;
#[path = "diagnostics_tests/snapshots.rs"]
mod snapshots;
