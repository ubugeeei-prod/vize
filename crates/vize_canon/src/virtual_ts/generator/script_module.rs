//! Module-scope facts collected from normal Vue `<script>` blocks.

mod namespace_hoist;
mod navigation;
mod plain_exports;

use super::imports::{IdentifierUsage, collect_identifier_usage};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Declaration, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, FxHashSet, String as VizeString};

pub(super) use namespace_hoist::NamespaceHoistPlan;
pub(super) use navigation::mapped_binding_range;
pub(super) use plain_exports::{
    collect_normal_script_named_value_exports,
    emit_setup_invocation_and_exports_with_mappings as emit_exports, push_setup_return_fields,
};

#[derive(Default)]
pub(super) struct ScriptModulePlan {
    pub(super) spans: Vec<(u32, u32)>,
    pub(super) exported_types: FxHashSet<CompactString>,
    pub(super) identifier_usage: IdentifierUsage,
}

impl ScriptModulePlan {
    pub(super) fn module_spans(
        &self,
        summary: &vize_croquis::Croquis,
        namespace_hoist: &NamespaceHoistPlan,
    ) -> Vec<(u32, u32)> {
        summary
            .import_statements
            .iter()
            .map(|import| (import.start, import.end))
            .chain(self.spans.iter().copied())
            .chain(namespace_hoist.spans().iter().copied())
            .chain(
                summary
                    .re_exports
                    .iter()
                    .map(|export| (export.start, export.end)),
            )
            .collect()
    }
}

pub(super) fn collect_script_module_plan(script: &str) -> ScriptModulePlan {
    let mut spans = Vec::new();
    let mut exported_types = FxHashSet::default();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts().with_module(true)).parse();
    let parsed = if parsed.panicked {
        Parser::new(&allocator, script, SourceType::tsx().with_module(true)).parse()
    } else {
        parsed
    };
    if parsed.panicked {
        return ScriptModulePlan::default();
    }

    for statement in &parsed.program.body {
        if let Statement::ExportNamedDeclaration(export) = statement {
            exported_types.extend(
                export
                    .specifiers
                    .iter()
                    .map(|specifier| CompactString::from(specifier.exported.name().as_str())),
            );
            let declared = export
                .declaration
                .as_ref()
                .and_then(|declaration| match declaration {
                    Declaration::TSTypeAliasDeclaration(declaration) => {
                        Some(declaration.id.name.as_str())
                    }
                    Declaration::TSInterfaceDeclaration(declaration) => {
                        Some(declaration.id.name.as_str())
                    }
                    Declaration::TSEnumDeclaration(declaration) => {
                        Some(declaration.id.name.as_str())
                    }
                    Declaration::ClassDeclaration(declaration) => {
                        declaration.id.as_ref().map(|id| id.name.as_str())
                    }
                    _ => None,
                });
            if let Some(name) = declared {
                exported_types.insert(name.into());
            }
        }
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
    ScriptModulePlan {
        spans: include_leading_ts_directive_comments(script, spans),
        exported_types,
        identifier_usage: collect_identifier_usage(&parsed.program),
    }
}

pub(super) fn collect_named_value_export_starts(script: &str) -> Vec<u32> {
    if !script.contains("export") {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts().with_module(true)).parse();
    let parsed = if parsed.panicked || !parsed.diagnostics.is_empty() {
        Parser::new(&allocator, script, SourceType::tsx().with_module(true)).parse()
    } else {
        parsed
    };
    if parsed.panicked {
        return Vec::new();
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

/// Erase only AST-proven value-export keywords. Spaces preserve every authored
/// byte/UTF-16 coordinate, including multiple statements or exports on one line.
pub(super) fn strip_named_value_exports(
    line: &mut std::borrow::Cow<'_, str>,
    line_start: usize,
    starts: &[u32],
) {
    let first = starts.partition_point(|&start| (start as usize) < line_start);
    for &start in &starts[first..] {
        let column = start as usize - line_start;
        if column >= line.len() {
            break;
        }
        if line.get(column..column + 6) == Some("export") {
            line.to_mut().replace_range(column..column + 6, "      ");
        }
    }
}

/// Emit the setup-scoped polyfill that avoids TS1343 under older module targets.
pub(super) fn emit_import_meta_polyfill(ts: &mut VizeString, script: &str) -> bool {
    let uses_import_meta = script.contains("import.meta");
    if uses_import_meta {
        ts.push_str("  const __import_meta: any = {};\n");
    }
    uses_import_meta
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

pub(super) fn include_leading_ts_directive_comments(
    script: &str,
    spans: Vec<(u32, u32)>,
) -> Vec<(u32, u32)> {
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
    use super::{
        collect_named_value_export_starts, collect_script_module_plan, strip_named_value_exports,
    };

    #[test]
    fn collect_import_span_includes_adjacent_ts_ignore_comment_group() {
        let script = "const before = 1;\n// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";\nconst after = 2;\n";
        let spans = collect_script_module_plan(script).spans;

        assert_eq!(spans.len(), 1);
        assert_eq!(
            &script[spans[0].0 as usize..spans[0].1 as usize],
            "// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";"
        );
    }

    #[test]
    fn collect_import_span_leaves_regular_comments_in_script_body() {
        let script = "// import note\nimport Chart from \"chart.js/auto/auto\";\n";
        let spans = collect_script_module_plan(script).spans;

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
    #[test]
    fn inline_exports_preserve_every_other_byte() {
        let script = "const text = 'export const x'; /* export */ export/*keep*/const café = 1; export const next = café;\nexport\nconst last = next;";
        let starts = collect_named_value_export_starts(script);
        assert_eq!(starts.len(), 3);
        let mut offset = 0;
        let mut lines = Vec::new();
        for line in script.split('\n') {
            let mut output = std::borrow::Cow::Borrowed(line);
            strip_named_value_exports(&mut output, offset, &starts);
            assert_eq!(output.len(), line.len());
            lines.push(output.into_owned());
            offset += line.len() + 1;
        }
        assert_eq!(
            lines.join("\n"),
            "const text = 'export const x'; /* export */       /*keep*/const café = 1;        const next = café;\n      \nconst last = next;"
        );
    }
}
