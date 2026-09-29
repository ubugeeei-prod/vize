use oxc_ast::ast::WithStatement;

use super::{BlockKind, BlockScopeData, GetSpan, ScriptParseResult};

pub(super) fn walk_with_statement(
    result: &mut ScriptParseResult,
    with_stmt: &WithStatement<'_>,
    source: &str,
) {
    super::walk_expression(result, &with_stmt.object, source);
    result.scopes.enter_block_scope(
        BlockScopeData {
            kind: BlockKind::With,
        },
        with_stmt.body.span().start,
        with_stmt.body.span().end,
    );
    super::walk_statement(result, &with_stmt.body, source);
    result.scopes.exit_scope();
}
