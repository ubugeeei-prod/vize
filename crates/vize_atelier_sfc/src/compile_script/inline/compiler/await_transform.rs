use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString};

use crate::module_map::Runs;

mod collector;
mod emit;

/// Transform top-level await expressions to use `_withAsyncContext`.
///
/// Recurses through setup statements and expressions, excluding nested functions.
/// The established direct-statement forms retain their emitted bytes and maps:
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

    let mut awaits = collector::SetupAwaits::default();
    awaits.visit_statements(&body.statements);
    emit::rewrite(source, &awaits.regions, AWAIT_WRAP_PREFIX.len(), runs)
}
