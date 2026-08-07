//! Module-scope hoisting of template-referenced literal consts.

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Expression, Statement, VariableDeclarationKind};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, ToCompactString};

use crate::script::ScriptCompileContext;

/// Separate hoisted consts (literal consts that can be module-level) from
/// setup code. Returns (hoisted_segments, setup_body_segments); a segment may
/// span multiple lines.
///
/// Selection is span-based on the parsed setup program: only a TOP-LEVEL
/// single-declarator `const` with an identifier binding, a literal
/// initializer, and a croquis `LiteralConst` binding of that name hoists. The
/// previous line-by-line scan matched by name alone, so a function-local
/// `const max = …` shadowing a hoistable top-level `const max = 7` was ripped
/// out of its function into module scope — a duplicate declaration referencing
/// setup bindings that do not exist there (#3944).
pub(super) fn separate_hoisted_consts(
    transformed_setup: &str,
    ctx: &ScriptCompileContext,
) -> (Vec<String>, Vec<String>) {
    let keep_everything = || {
        (
            Vec::new(),
            transformed_setup
                .lines()
                .map(|line| line.to_compact_string())
                .collect(),
        )
    };

    let allocator = Allocator::default();
    // TS parses a superset of the transformed setup in either lang.
    let source_type = SourceType::ts();
    let parsed = Parser::new(&allocator, transformed_setup, source_type).parse();
    // `panicked` alone only covers an unrecoverable abort: oxc also returns a
    // recovered program with diagnostics attached, and the statements it
    // salvaged around a syntax error are not a trustworthy basis for moving
    // declarations between scopes. Either way, hoist nothing rather than guess.
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return keep_everything();
    }

    let mut hoisted: Vec<String> = Vec::new();
    let mut body: Vec<String> = Vec::new();
    let mut prev_end = 0usize;

    for statement in &parsed.program.body {
        let span = statement.span();
        let (start, end) = (span.start as usize, span.end as usize);
        if start < prev_end || end > transformed_setup.len() || start > end {
            return keep_everything();
        }
        // Gaps (blank lines, comments) stay with the setup body in order.
        let gap = &transformed_setup[prev_end..start];
        if !gap.trim().is_empty() {
            body.push(gap.trim_matches('\n').into());
        }
        let slice: String = transformed_setup[start..end].into();
        if is_hoistable_literal_const(statement, ctx) {
            hoisted.push(slice);
        } else {
            body.push(slice);
        }
        prev_end = end;
    }
    let tail = &transformed_setup[prev_end..];
    if !tail.trim().is_empty() {
        body.push(tail.trim_matches('\n').into());
    }

    (hoisted, body)
}

/// A top-level `const <ident> = <literal>` whose name croquis classified as
/// `LiteralConst`. The initializer check keeps rewritten or computed values in
/// setup scope even when the analysis says the binding is literal-like.
fn is_hoistable_literal_const(statement: &Statement<'_>, ctx: &ScriptCompileContext) -> bool {
    let Statement::VariableDeclaration(declaration) = statement else {
        return false;
    };
    if declaration.kind != VariableDeclarationKind::Const || declaration.declarations.len() != 1 {
        return false;
    }
    let declarator = &declaration.declarations[0];
    let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
        return false;
    };
    let initializer_is_literal = matches!(
        declarator.init,
        Some(
            Expression::NumericLiteral(_)
                | Expression::StringLiteral(_)
                | Expression::BooleanLiteral(_)
                | Expression::BigIntLiteral(_)
                | Expression::NullLiteral(_)
        )
    );
    initializer_is_literal
        && matches!(
            ctx.bindings.bindings.get(identifier.name.as_str()),
            Some(crate::types::BindingType::LiteralConst)
        )
}
