use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, ToCompactString};

use crate::module_map::Runs;

/// Transform top-level await expressions to use `_withAsyncContext`.
///
/// Handles two patterns:
/// 1. `const x = await expr` → `const x = (\n  ([__temp,__restore] = _withAsyncContext(() => expr)),\n  __temp = await __temp,\n  __restore(),\n  __temp\n)`
/// 2. `await expr` (statement) → `;(\n  ([__temp,__restore] = _withAsyncContext(() => expr)),\n  await __temp,\n  __restore()\n)`
///
/// Given each input line's provenance (parallel to `lines`), each output
/// line's provenance is returned too (Davinci P3-9 source maps); a rewritten
/// await statement is anchored at the statement it replaced.
pub(super) fn transform_await_expressions(
    lines: &[String],
    line_runs: Option<&[Runs]>,
    is_ts: bool,
) -> (Vec<String>, Vec<Runs>) {
    let mut source = String::default();
    let mut source_runs = Runs::default();
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            source.push('\n');
        }
        if let Some(runs) = line_runs.and_then(|runs| runs.get(idx)) {
            source_runs.append(source.len(), runs);
        }
        source.push_str(line);
    }

    let mut runs = Runs::default();
    let transformed = transform_await_source(&source, is_ts, &mut runs);
    let composed = runs.compose(&source_runs);
    let base = transformed.as_ptr() as usize;
    let mut output_runs = Vec::new();
    let output = transformed
        .lines()
        .map(|line| {
            if line_runs.is_some() {
                let offset = line.as_ptr() as usize - base;
                output_runs.push(composed.slice(offset, line.len()));
            }
            line.to_compact_string()
        })
        .collect();
    (output, output_runs)
}

const AWAIT_WRAP_PREFIX: &str = "async function __vize_async_setup__() {\n";
const AWAIT_WRAP_SUFFIX: &str = "\n}";

/// Rewrite the top-level awaits of `source`, recording the result's
/// provenance in `source` offsets into `runs`.
fn transform_await_source(source: &str, is_ts: bool, runs: &mut Runs) -> String {
    match rewrite_awaits(source, is_ts, runs) {
        Some(transformed) => transformed,
        None => {
            *runs = Runs::identity(source.len());
            source.to_compact_string()
        }
    }
}

/// The rewritten source, or `None` when `source` is kept as it is.
fn rewrite_awaits(source: &str, is_ts: bool, runs: &mut Runs) -> Option<String> {
    if source.trim().is_empty() {
        return None;
    }

    let mut wrapped =
        String::with_capacity(AWAIT_WRAP_PREFIX.len() + source.len() + AWAIT_WRAP_SUFFIX.len());
    wrapped.push_str(AWAIT_WRAP_PREFIX);
    wrapped.push_str(source);
    wrapped.push_str(AWAIT_WRAP_SUFFIX);

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(is_ts);
    let parse_result = Parser::new(&allocator, &wrapped, source_type).parse();
    if !parse_result.diagnostics.is_empty() {
        return None;
    }

    let Some(Statement::FunctionDeclaration(func)) = parse_result.program.body.first() else {
        return None;
    };
    let Some(body) = &func.body else {
        return None;
    };

    let offset = AWAIT_WRAP_PREFIX.len();
    let mut cursor = 0usize;
    let mut transformed = String::with_capacity(source.len() + 128);

    for stmt in body.statements.iter() {
        let stmt_span = stmt.span();
        let stmt_start = stmt_span.start.try_into().ok().and_then(|start: usize| {
            start
                .checked_sub(offset)
                .filter(|start| *start <= source.len())
        })?;
        let stmt_end =
            stmt_span.end.try_into().ok().and_then(|end: usize| {
                end.checked_sub(offset).filter(|end| *end <= source.len())
            })?;

        if stmt_start < cursor || stmt_start > stmt_end {
            return None;
        }

        runs.copy(transformed.len(), cursor, stmt_start - cursor);
        transformed.push_str(&source[cursor..stmt_start]);

        if let Some(replacement) = transform_await_statement(source, stmt, offset) {
            runs.point(transformed.len(), stmt_start);
            transformed.push_str(&replacement);
        } else {
            runs.copy(transformed.len(), stmt_start, stmt_end - stmt_start);
            transformed.push_str(&source[stmt_start..stmt_end]);
        }

        cursor = stmt_end;
    }

    runs.copy(transformed.len(), cursor, source.len() - cursor);
    transformed.push_str(&source[cursor..]);
    Some(transformed)
}

fn transform_await_statement(source: &str, stmt: &Statement<'_>, offset: usize) -> Option<String> {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            let Expression::AwaitExpression(await_expr) = &expr_stmt.expression else {
                return None;
            };
            build_standalone_await_replacement(source, stmt.span(), await_expr.span(), offset)
        }
        Statement::VariableDeclaration(var_decl) => {
            if var_decl.declarations.len() != 1 {
                return None;
            }
            let declarator = var_decl.declarations.first()?;
            let init = declarator.init.as_ref()?;
            let Expression::AwaitExpression(await_expr) = init else {
                return None;
            };
            build_await_assignment_replacement(source, stmt.span(), await_expr.span(), offset)
        }
        _ => None,
    }
}

fn build_await_assignment_replacement(
    source: &str,
    stmt_span: oxc_span::Span,
    await_span: oxc_span::Span,
    offset: usize,
) -> Option<String> {
    let stmt_start = stmt_span.start as usize - offset;
    let stmt_end = stmt_span.end as usize - offset;
    let await_start = await_span.start as usize - offset;
    let await_end = await_span.end as usize - offset;

    let prefix = source.get(stmt_start..await_start)?;
    let expr = await_expression_source(source, await_start, await_end)?;
    let suffix = source.get(await_end..stmt_end)?;

    let mut out = String::with_capacity(prefix.len() + expr.len() + suffix.len() + 96);
    out.push_str(prefix);
    out.push_str(" (\n");
    out.push_str("  ([__temp,__restore] = _withAsyncContext(() => ");
    out.push_str(expr);
    out.push_str(")),\n");
    out.push_str("  __temp = await __temp,\n");
    out.push_str("  __restore(),\n");
    out.push_str("  __temp\n");
    out.push(')');
    out.push_str(suffix);
    Some(out)
}

fn build_standalone_await_replacement(
    source: &str,
    stmt_span: oxc_span::Span,
    await_span: oxc_span::Span,
    offset: usize,
) -> Option<String> {
    let stmt_start = stmt_span.start as usize - offset;
    let stmt_end = stmt_span.end as usize - offset;
    let await_start = await_span.start as usize - offset;
    let await_end = await_span.end as usize - offset;

    if stmt_start != await_start {
        return None;
    }

    let expr = await_expression_source(source, await_start, await_end)?;
    let suffix = source.get(await_end..stmt_end)?;

    let mut out = String::with_capacity(expr.len() + suffix.len() + 72);
    out.push_str(";(\n");
    out.push_str("  ([__temp,__restore] = _withAsyncContext(() => ");
    out.push_str(expr);
    out.push_str(")),\n");
    out.push_str("  await __temp,\n");
    out.push_str("  __restore()\n");
    out.push(')');
    out.push_str(suffix);
    Some(out)
}

fn await_expression_source(source: &str, start: usize, end: usize) -> Option<&str> {
    let await_source = source.get(start..end)?;
    let expr = await_source.strip_prefix("await")?.trim_start();
    if expr.is_empty() {
        return None;
    }
    Some(expr)
}
