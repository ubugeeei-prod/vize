//! Module-scope facts collected from normal Vue `<script>` blocks.

mod plain_exports;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Statement, TSEnumDeclaration};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, FxHashSet, String as VizeString};

pub(super) use plain_exports::{
    collect_normal_script_named_value_exports, emit_setup_invocation_and_exports,
    push_setup_return_fields,
};

pub(super) fn collect_line_module_spans(script: &str) -> Vec<(u32, u32)> {
    let mut spans = Vec::new();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts().with_module(true)).parse();
    let parsed = if parsed.panicked {
        Parser::new(&allocator, script, SourceType::tsx().with_module(true)).parse()
    } else {
        parsed
    };
    if parsed.panicked {
        return spans;
    }

    for statement in &parsed.program.body {
        match statement {
            Statement::ImportDeclaration(_)
            | Statement::ExportAllDeclaration(_)
            | Statement::TSModuleDeclaration(_)
            | Statement::TSGlobalDeclaration(_) => {
                let span = statement.span();
                spans.push((span.start, span.end));
            }
            Statement::ExportNamedDeclaration(decl) if decl.source.is_some() => {
                spans.push((decl.span.start, decl.span.end));
            }
            _ => {}
        }
    }
    include_leading_ts_directive_comments(script, spans)
}

pub(super) fn collect_named_value_export_starts(script: &str) -> FxHashSet<u32> {
    if !script.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("export ")
            && !line.starts_with("export default")
            && !line.starts_with("export type ")
            && !line.starts_with("export interface ")
    }) {
        return FxHashSet::default();
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts().with_module(true)).parse();
    let parsed = if parsed.panicked || !parsed.errors.is_empty() {
        Parser::new(&allocator, script, SourceType::tsx().with_module(true)).parse()
    } else {
        parsed
    };
    if parsed.panicked {
        return FxHashSet::default();
    }

    parsed
        .program
        .body
        .iter()
        .filter_map(|statement| {
            let Statement::ExportNamedDeclaration(export) = statement else {
                return None;
            };
            export
                .declaration
                .as_ref()
                .is_some_and(declaration_has_runtime_value)
                .then_some(export.span.start)
        })
        .collect()
}

/// Emit the setup-scoped polyfill that avoids TS1343 under older module targets.
pub(super) fn emit_import_meta_polyfill(ts: &mut VizeString, script: &str) -> bool {
    let uses_import_meta = script.contains("import.meta");
    if uses_import_meta {
        ts.push_str("  const __import_meta: any = {};\n");
    }
    uses_import_meta
}

pub(super) fn collect_const_enum_names(script: &str) -> FxHashSet<CompactString> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return FxHashSet::default();
    }

    let mut collector = ConstEnumNames::default();
    collector.visit_program(&parsed.program);
    collector.names
}

#[derive(Default)]
struct ConstEnumNames {
    names: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for ConstEnumNames {
    fn visit_ts_enum_declaration(&mut self, decl: &TSEnumDeclaration<'a>) {
        if decl.r#const {
            self.names.insert(CompactString::new(decl.id.name.as_str()));
        }
    }
}

fn declaration_has_runtime_value(declaration: &Declaration<'_>) -> bool {
    matches!(
        declaration,
        Declaration::VariableDeclaration(_)
            | Declaration::FunctionDeclaration(_)
            | Declaration::ClassDeclaration(_)
            | Declaration::TSEnumDeclaration(_)
    )
}

fn include_leading_ts_directive_comments(script: &str, spans: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    spans
        .into_iter()
        .map(|(start, end)| {
            let start = leading_ts_directive_comment_start(script, start as usize)
                .unwrap_or(start as usize);
            (start as u32, end)
        })
        .collect()
}

fn leading_ts_directive_comment_start(script: &str, statement_start: usize) -> Option<usize> {
    let mut cursor = line_start_at(script, statement_start);
    let mut comment_group_start = None;
    let mut has_ts_directive = false;
    while cursor > 0 {
        let previous_line_end = cursor.saturating_sub(1);
        let previous_line_start = script[..previous_line_end]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        let line = &script[previous_line_start..previous_line_end];
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = line.trim_start();
        if !trimmed.starts_with("//") {
            break;
        }
        comment_group_start = Some(previous_line_start);
        if contains_ts_suppression_directive(trimmed) {
            has_ts_directive = true;
        }
        cursor = previous_line_start;
    }
    has_ts_directive.then_some(comment_group_start).flatten()
}

fn line_start_at(script: &str, offset: usize) -> usize {
    script[..offset.min(script.len())]
        .rfind('\n')
        .map_or(0, |index| index + 1)
}

fn contains_ts_suppression_directive(comment: &str) -> bool {
    comment.contains("@ts-ignore") || comment.contains("@ts-expect-error")
}

#[cfg(test)]
mod tests {
    use super::{collect_line_module_spans, collect_named_value_export_starts};

    #[test]
    fn collect_import_span_includes_adjacent_ts_ignore_comment_group() {
        let script = "const before = 1;\n// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";\nconst after = 2;\n";
        let spans = collect_line_module_spans(script);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            &script[spans[0].0 as usize..spans[0].1 as usize],
            "// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";"
        );
    }

    #[test]
    fn collect_import_span_leaves_regular_comments_in_script_body() {
        let script = "// import note\nimport Chart from \"chart.js/auto/auto\";\n";
        let spans = collect_line_module_spans(script);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            &script[spans[0].0 as usize..spans[0].1 as usize],
            "import Chart from \"chart.js/auto/auto\";"
        );
    }

    #[test]
    fn named_value_export_starts_exclude_nested_enum_members() {
        let script =
            "enum Modes {\n  export = 'export',\n}\nexport const selected = Modes.export;\n";
        let starts = collect_named_value_export_starts(script);

        assert_eq!(starts.len(), 1);
        assert!(starts.contains(&(script.find("export const").unwrap() as u32)));
        assert!(!starts.contains(&(script.find("export =").unwrap() as u32)));
    }
}
