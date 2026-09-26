//! An accepted L3 artifact completes here, including invariant failures.

use crate::compile::VaporCompileResult;
use crate::generate::spans::VaporSourceSpans;
use crate::ir::RootIRNode;
use crate::l3::{self, VaporL3Artifact};
use vize_carton::{Allocator, String};

pub(crate) fn emit_accepted<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    artifact: VaporL3Artifact<'a>,
    scope_id: Option<&str>,
    source_map: bool,
    emit: impl FnOnce(&RootIRNode<'a>, Option<&VaporSourceSpans>) -> VaporCompileResult,
) -> VaporCompileResult {
    match artifact.into_ir_with_spans(allocator, source, scope_id, source_map) {
        Some((ir, spans)) => {
            l3::record_accepted();
            emit(&ir, spans.as_ref())
        }
        None => {
            l3::record_rejected();
            VaporCompileResult {
                code: String::default(),
                templates: Vec::new(),
                map: None,
                error_messages: vec![String::from(
                    "Davinci L3 emission rejected Vapor artifact: inconsistent checked payload",
                )],
            }
        }
    }
}
