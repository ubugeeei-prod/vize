use super::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::FileId;
use vize_carton::append;

fn make_file_id() -> FileId {
    FileId::new(0)
}

mod codes;
mod markdown;
mod snapshots;
