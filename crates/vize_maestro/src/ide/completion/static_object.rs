//! Pre-Corsa completion for exact local object literals.

use tower_lsp::lsp_types::CompletionResponse;

use super::{CompletionService, script};
use crate::ide::IdeContext;
use crate::virtual_code::BlockType;

impl CompletionService {
    /// Return a source-local object member list only when it is complete and
    /// scope-exact. This avoids initializing Corsa for information already
    /// proven by the authored AST.
    pub(crate) fn complete_static_object_member(
        ctx: &IdeContext<'_>,
    ) -> Option<CompletionResponse> {
        let items = match ctx.block_type? {
            BlockType::Script => script::complete_static_object_member_access(ctx, false),
            BlockType::ScriptSetup => script::complete_static_object_member_access(ctx, true),
            _ => None,
        }?;
        Some(CompletionResponse::Array(items))
    }
}
