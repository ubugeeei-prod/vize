//! AST-based top-level statement extraction for script setup compilation.
//!
//! This keeps imports, TypeScript declarations, and setup code separated using
//! precise OXC statement spans instead of line-based heuristics.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Expression, Program, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use super::macros::is_runtime_erased_macro;
use vize_carton::{FxHashSet, String, ToCompactString};
use vize_croquis::macros::is_builtin_macro;

use crate::module_map::Runs;
use crate::script::is_static_enum;

use super::runtime_bindings::collect_runtime_bindings;

/// Where each extracted section came from, in `content` offsets (Davinci P3-9).
#[derive(Debug, Default)]
pub(crate) struct SectionTrace {
    /// The provenance of each setup line, parallel to the setup lines.
    pub(crate) setup_lines: Vec<Runs>,
    /// The provenance of each user import's text, and the offset of the
    /// import statement it came from, parallel to the imports.
    pub(crate) imports: Vec<(Runs, usize)>,
    /// The provenance of each preserved declaration, parallel to them.
    pub(crate) ts_declarations: Vec<Runs>,
    /// The `start..end` of every macro statement the sections drop.
    pub(crate) macros: Vec<(usize, usize)>,
}

enum StatementBucket {
    Import,
    TypeDeclaration,
    HoistedRuntime,
    Macro,
    Setup,
}

pub(crate) fn extract_script_sections(
    content: &str,
    is_ts: bool,
) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, content, source_type).parse();

    if ret.panicked {
        return None;
    }

    extract_script_sections_from_program(&ret.program, content, is_ts)
}

/// Parse-free core of [`extract_script_sections`] for callers that already
/// hold an oxc `Program` for `content` (the SFC compiler's parse-once
/// pipeline). `content` must be the exact text the program was parsed from.
pub(crate) fn extract_script_sections_from_program(
    program: &Program<'_>,
    content: &str,
    is_ts: bool,
) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
    extract_script_sections_from_program_with_options(program, content, is_ts, false)
}

pub(crate) fn extract_script_sections_from_program_with_options(
    program: &Program<'_>,
    content: &str,
    is_ts: bool,
    preserve_runtime_erased_macros: bool,
) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
    extract_sections(
        program,
        content,
        is_ts,
        preserve_runtime_erased_macros,
        None,
    )
}

/// [`extract_script_sections_from_program_with_options`] that also records
/// where every section came from. The sections are byte-identical to the
/// untraced call's.
pub(crate) fn extract_script_sections_traced(
    program: &Program<'_>,
    content: &str,
    is_ts: bool,
    preserve_runtime_erased_macros: bool,
    trace: &mut SectionTrace,
) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
    extract_sections(
        program,
        content,
        is_ts,
        preserve_runtime_erased_macros,
        Some(trace),
    )
}

fn extract_sections(
    program: &Program<'_>,
    content: &str,
    is_ts: bool,
    preserve_runtime_erased_macros: bool,
    mut trace: Option<&mut SectionTrace>,
) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
    let mut user_imports = Vec::new();
    let mut setup_lines = Vec::new();
    let mut ts_declarations = Vec::new();

    let mut prev_end = 0usize;
    let mut pending_gap = String::default();
    // The gap accumulates across dropped macro statements, so it is not one
    // contiguous slice of `content`; its provenance is kept beside it.
    let mut pending_gap_runs = Runs::default();
    let runtime_bindings = collect_runtime_bindings(program.body.iter());

    for stmt in program.body.iter() {
        let span = stmt.span();
        let start = span.start as usize;
        let end = span.end as usize;

        let (Some(gap), Some(slice)) = (content.get(prev_end..start), content.get(start..end))
        else {
            return None;
        };

        pending_gap_runs.copy(pending_gap.len(), prev_end, start - prev_end);
        pending_gap.push_str(gap);

        let bucket = classify_statement(
            stmt,
            slice,
            &runtime_bindings,
            preserve_runtime_erased_macros,
        );
        if matches!(bucket, StatementBucket::Macro) {
            if let Some(trace) = trace.as_deref_mut() {
                trace.macros.push((start, end));
            }
            prev_end = end;
            continue;
        }
        let mut segment = std::mem::take(&mut pending_gap);
        let mut segment_runs = std::mem::take(&mut pending_gap_runs);
        segment_runs.copy(segment.len(), start, slice.len());
        segment.push_str(slice);
        match bucket {
            StatementBucket::Import => {
                user_imports.push(normalize_statement_segment(&segment));
                if let Some(trace) = trace.as_deref_mut() {
                    let runs = preserved_segment_runs(&segment, &segment_runs);
                    trace.imports.push((runs, start));
                }
            }
            StatementBucket::TypeDeclaration | StatementBucket::HoistedRuntime => {
                if is_ts || matches!(bucket, StatementBucket::HoistedRuntime) {
                    ts_declarations.push(normalize_preserved_segment(&segment));
                    if let Some(trace) = trace.as_deref_mut() {
                        let runs = preserved_segment_runs(&segment, &segment_runs);
                        trace.ts_declarations.push(runs);
                    }
                }
            }
            StatementBucket::Setup => {
                push_non_empty_lines(&mut setup_lines, &segment);
                if let Some(trace) = trace.as_deref_mut() {
                    trace_non_empty_lines(&mut trace.setup_lines, &segment, &segment_runs);
                }
            }
            // Macro statements were skipped before the segment was taken.
            StatementBucket::Macro => {}
        }

        prev_end = end;
    }

    pending_gap_runs.copy(pending_gap.len(), prev_end, content.len() - prev_end);
    pending_gap.push_str(content.get(prev_end..).unwrap_or_default());
    if !pending_gap.trim().is_empty() {
        push_non_empty_lines(&mut setup_lines, &pending_gap);
        if let Some(trace) = trace {
            trace_non_empty_lines(&mut trace.setup_lines, &pending_gap, &pending_gap_runs);
        }
    }

    Some((user_imports, setup_lines, ts_declarations))
}

fn classify_statement(
    stmt: &Statement<'_>,
    slice: &str,
    runtime_bindings: &FxHashSet<String>,
    preserve_runtime_erased_macros: bool,
) -> StatementBucket {
    let trimmed = slice.trim_start();

    if trimmed.starts_with("declare ") {
        return StatementBucket::TypeDeclaration;
    }

    match stmt {
        Statement::ImportDeclaration(_) => StatementBucket::Import,
        Statement::TSInterfaceDeclaration(_) | Statement::TSTypeAliasDeclaration(_) => {
            StatementBucket::TypeDeclaration
        }
        Statement::TSEnumDeclaration(declaration) if is_static_enum(declaration) => {
            StatementBucket::HoistedRuntime
        }
        Statement::ExportNamedDeclaration(export_decl) => {
            if export_decl.export_kind.is_type()
                || export_decl.declaration.as_ref().is_some_and(|decl| {
                    matches!(
                        decl,
                        Declaration::TSInterfaceDeclaration(_)
                            | Declaration::TSTypeAliasDeclaration(_)
                    )
                })
                || trimmed.starts_with("export type ")
                || trimmed.starts_with("export interface ")
            {
                StatementBucket::TypeDeclaration
            } else {
                StatementBucket::Setup
            }
        }
        Statement::ExpressionStatement(expr_stmt) => {
            if unwrap_call_expression(&expr_stmt.expression).is_some_and(|call| {
                is_macro_call(call, runtime_bindings, preserve_runtime_erased_macros)
            }) {
                StatementBucket::Macro
            } else {
                StatementBucket::Setup
            }
        }
        Statement::VariableDeclaration(var_decl) => {
            if var_decl.declarations.iter().any(|decl| {
                decl.init
                    .as_ref()
                    .and_then(unwrap_call_expression)
                    .is_some_and(|call| {
                        is_macro_call(call, runtime_bindings, preserve_runtime_erased_macros)
                    })
            }) {
                StatementBucket::Macro
            } else {
                StatementBucket::Setup
            }
        }
        _ => StatementBucket::Setup,
    }
}

fn unwrap_call_expression<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::TSAsExpression(ts_as) => unwrap_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            unwrap_call_expression(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(paren) => unwrap_call_expression(&paren.expression),
        _ => None,
    }
}

fn is_macro_call(
    call: &oxc_ast::ast::CallExpression<'_>,
    runtime_bindings: &FxHashSet<String>,
    preserve_runtime_erased_macros: bool,
) -> bool {
    match &call.callee {
        Expression::Identifier(id) => {
            let name = id.name.as_str();
            is_builtin_macro(name)
                || (!preserve_runtime_erased_macros
                    && is_runtime_erased_macro(name)
                    && !runtime_bindings.contains(name))
        }
        _ => false,
    }
}

fn normalize_statement_segment(segment: &str) -> String {
    let trimmed = normalize_preserved_segment(segment);
    let mut normalized = trimmed;
    normalized.push('\n');
    normalized
}

fn normalize_preserved_segment(segment: &str) -> String {
    segment
        .trim_start_matches(['\n', '\r'])
        .trim_end_matches(['\n', '\r'])
        .to_compact_string()
}

fn push_non_empty_lines(lines: &mut Vec<String>, segment: &str) {
    for line in segment.lines() {
        if !line.trim().is_empty() {
            lines.push(line.to_compact_string());
        }
    }
}

/// The provenance of [`normalize_preserved_segment`]'s result (the newline
/// [`normalize_statement_segment`] appends is not a copy).
fn preserved_segment_runs(segment: &str, runs: &Runs) -> Runs {
    let lead = segment.len() - segment.trim_start_matches(['\n', '\r']).len();
    let kept = segment
        .get(lead..)
        .unwrap_or_default()
        .trim_end_matches(['\n', '\r'])
        .len();
    runs.slice(lead, kept)
}

/// The provenance of each line [`push_non_empty_lines`] keeps, in order.
fn trace_non_empty_lines(lines: &mut Vec<Runs>, segment: &str, runs: &Runs) {
    for line in segment.lines() {
        if !line.trim().is_empty() {
            let offset = line.as_ptr() as usize - segment.as_ptr() as usize;
            lines.push(runs.slice(offset, line.len()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::extract_script_sections;

    #[test]
    fn skips_next_line_macro_assignment() {
        let content = r#"const props =
  defineProps<{
    name: string
  }>()

const count = 1
"#;
        let (_, setup, module) = extract_script_sections(content, true).unwrap();

        assert_eq!(setup.as_slice(), ["const count = 1"]);
        assert!(module.is_empty());
    }

    #[test]
    fn trims_leading_blank_lines_from_type_declarations() {
        let content = r#"import { useStore } from 'vuex'

interface RootState {
  count: number
}

const store = useStore<RootState>()
"#;
        let (_, _, module) = extract_script_sections(content, true).unwrap();

        assert_eq!(
            module.as_slice(),
            ["interface RootState {\n  count: number\n}"]
        );
    }

    #[test]
    fn hoists_only_static_runtime_enums() {
        let content = r#"const seed = runtime()

enum StaticStep { Name, General = 1 + 1 }
enum DynamicStep { Name = seed }
"#;
        let (_, setup, module) = extract_script_sections(content, false).unwrap();

        assert_eq!(
            module.as_slice(),
            ["enum StaticStep { Name, General = 1 + 1 }"]
        );
        assert_eq!(
            setup.as_slice(),
            ["const seed = runtime()", "enum DynamicStep { Name = seed }"]
        );
    }

    #[test]
    fn skips_ecosystem_compile_time_macro() {
        let content = r#"definePage({
  name: 'home',
  meta: {
    requiresAuth: true,
  },
})

const msg = 'ready'
"#;
        let (_, setup, module) = extract_script_sections(content, true).unwrap();

        assert_eq!(setup.as_slice(), ["const msg = 'ready'"]);
        assert!(module.is_empty());
    }

    #[test]
    fn preserves_imported_define_page() {
        let content = r#"import { definePage } from '@/page.js'

definePage(() => ({
  title: 'runtime page',
}))

const msg = 'ready'
"#;
        let (imports, setup, module) = extract_script_sections(content, true).unwrap();

        assert_eq!(imports.len(), 1);
        assert_eq!(
            setup,
            [
                "definePage(() => ({",
                "  title: 'runtime page',",
                "}))",
                "const msg = 'ready'"
            ]
        );
        assert!(module.is_empty());
    }

    #[test]
    fn preserves_define_page_meta_outside_nuxt() {
        let content = r#"definePageMeta({
  name: 'docs',
  meta: {
    scrollMargin: 180,
  },
})

const msg = 'ready'
"#;
        let (_, setup, module) = extract_script_sections(content, true).unwrap();

        assert_eq!(
            setup,
            content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .collect::<Vec<_>>()
        );
        assert!(module.is_empty());
    }

    #[test]
    fn skips_define_route_rules() {
        let content = r#"defineRouteRules({
  prerender: true,
  cache: {
    maxAge: 60,
  },
})

const msg = 'ready'
"#;
        let (_, setup, module) = extract_script_sections(content, true).unwrap();

        assert_eq!(setup.as_slice(), ["const msg = 'ready'"]);
        assert!(module.is_empty());
    }
}
