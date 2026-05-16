use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, ToCompactString};

/// Transform top-level await expressions to use `_withAsyncContext`.
///
/// Handles two patterns:
/// 1. `const x = await expr` → `const x = (\n  ([__temp,__restore] = _withAsyncContext(() => expr)),\n  __temp = await __temp,\n  __restore(),\n  __temp\n)`
/// 2. `await expr` (statement) → `;(\n  ([__temp,__restore] = _withAsyncContext(() => expr)),\n  await __temp,\n  __restore()\n)`
pub(super) fn transform_await_expressions(lines: &[String], is_ts: bool) -> Vec<String> {
    let mut source = String::default();
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            source.push('\n');
        }
        source.push_str(line);
    }

    transform_await_source(&source, is_ts)
        .lines()
        .map(|line| line.to_compact_string())
        .collect()
}

const AWAIT_WRAP_PREFIX: &str = "async function __vize_async_setup__() {\n";
const AWAIT_WRAP_SUFFIX: &str = "\n}";

fn transform_await_source(source: &str, is_ts: bool) -> String {
    if source.trim().is_empty() {
        return source.to_compact_string();
    }

    let mut wrapped =
        String::with_capacity(AWAIT_WRAP_PREFIX.len() + source.len() + AWAIT_WRAP_SUFFIX.len());
    wrapped.push_str(AWAIT_WRAP_PREFIX);
    wrapped.push_str(source);
    wrapped.push_str(AWAIT_WRAP_SUFFIX);

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(is_ts);
    let parse_result = Parser::new(&allocator, &wrapped, source_type).parse();
    if !parse_result.errors.is_empty() {
        return source.to_compact_string();
    }

    let Some(Statement::FunctionDeclaration(func)) = parse_result.program.body.first() else {
        return source.to_compact_string();
    };
    let Some(body) = &func.body else {
        return source.to_compact_string();
    };

    let offset = AWAIT_WRAP_PREFIX.len();
    let mut cursor = 0usize;
    let mut transformed = String::with_capacity(source.len() + 128);

    for stmt in body.statements.iter() {
        let stmt_span = stmt.span();
        let Some(stmt_start) = stmt_span.start.try_into().ok().and_then(|start: usize| {
            start
                .checked_sub(offset)
                .filter(|start| *start <= source.len())
        }) else {
            return source.to_compact_string();
        };
        let Some(stmt_end) = stmt_span
            .end
            .try_into()
            .ok()
            .and_then(|end: usize| end.checked_sub(offset).filter(|end| *end <= source.len()))
        else {
            return source.to_compact_string();
        };

        if stmt_start < cursor || stmt_start > stmt_end {
            return source.to_compact_string();
        }

        transformed.push_str(&source[cursor..stmt_start]);

        if let Some(replacement) = transform_await_statement(source, stmt, offset) {
            transformed.push_str(&replacement);
        } else {
            transformed.push_str(&source[stmt_start..stmt_end]);
        }

        cursor = stmt_end;
    }

    transformed.push_str(&source[cursor..]);
    transformed
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
