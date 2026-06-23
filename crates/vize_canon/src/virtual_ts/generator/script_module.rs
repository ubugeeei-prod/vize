//! Module-scope facts collected from normal Vue `<script>` blocks.

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Declaration, Statement, TSEnumDeclaration};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String as VizeString, append};

pub(super) fn collect_normal_script_named_value_exports(
    script: Option<&str>,
    has_script_setup: bool,
    has_plain_script_scope: bool,
) -> Vec<CompactString> {
    if has_script_setup || !has_plain_script_scope {
        return Vec::new();
    }
    script.map(collect_named_value_exports).unwrap_or_default()
}

pub(super) fn collect_line_module_import_spans(script: &str) -> Vec<(u32, u32)> {
    let mut spans = Vec::new();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::tsx().with_module(true)).parse();
    if parsed.panicked {
        return spans;
    }

    for statement in &parsed.program.body {
        match statement {
            Statement::ImportDeclaration(decl) => {
                spans.push((decl.span.start, decl.span.end));
            }
            Statement::ExportNamedDeclaration(decl) if decl.source.is_some() => {
                spans.push((decl.span.start, decl.span.end));
            }
            Statement::ExportAllDeclaration(decl) => {
                spans.push((decl.span.start, decl.span.end));
            }
            _ => {}
        }
    }
    include_leading_ts_directive_comments(script, spans)
}

pub(super) fn push_setup_return_fields(names: &[CompactString], fields: &mut Vec<CompactString>) {
    fields.extend(names.iter().cloned());
}

pub(super) fn emit_setup_invocation_and_exports(ts: &mut VizeString, names: &[CompactString]) {
    if names.is_empty() {
        ts.push_str("__setup();\n\n");
        return;
    }

    ts.push_str("const __vize_plain_script_exports = __setup();\n");
    for name in names {
        append!(
            *ts,
            "export const {name} = __vize_plain_script_exports.{name};\n"
        );
    }
    ts.push('\n');
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

fn collect_named_value_exports(script: &str) -> Vec<CompactString> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return Vec::new();
    }

    let mut seen = FxHashSet::default();
    let mut names = Vec::new();
    for statement in &parsed.program.body {
        collect_statement_exports(statement, &mut seen, &mut names);
    }
    names
}

fn collect_statement_exports(
    statement: &Statement<'_>,
    seen: &mut FxHashSet<CompactString>,
    names: &mut Vec<CompactString>,
) {
    let Statement::ExportNamedDeclaration(export) = statement else {
        return;
    };
    if export.source.is_some() || export.export_kind.is_type() {
        return;
    }
    let Some(declaration) = export.declaration.as_ref() else {
        return;
    };
    collect_declaration_exports(declaration, seen, names);
}

fn collect_declaration_exports(
    declaration: &Declaration<'_>,
    seen: &mut FxHashSet<CompactString>,
    names: &mut Vec<CompactString>,
) {
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                collect_binding_names(&declarator.id, seen, names);
            }
        }
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id {
                push_name(id.name.as_str(), seen, names);
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                push_name(id.name.as_str(), seen, names);
            }
        }
        _ => {}
    }
}

fn collect_binding_names(
    pattern: &BindingPattern<'_>,
    seen: &mut FxHashSet<CompactString>,
    names: &mut Vec<CompactString>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => push_name(id.name.as_str(), seen, names),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_binding_names(&property.value, seen, names);
            }
            if let Some(rest) = &object.rest {
                collect_binding_names(&rest.argument, seen, names);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_binding_names(element, seen, names);
            }
            if let Some(rest) = &array.rest {
                collect_binding_names(&rest.argument, seen, names);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_binding_names(&assignment.left, seen, names);
        }
    }
}

fn push_name(name: &str, seen: &mut FxHashSet<CompactString>, names: &mut Vec<CompactString>) {
    let name = CompactString::new(name);
    if seen.insert(name.clone()) {
        names.push(name);
    }
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
    use super::collect_line_module_import_spans;

    #[test]
    fn collect_import_span_includes_adjacent_ts_ignore_comment_group() {
        let script = "const before = 1;\n// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";\nconst after = 2;\n";
        let spans = collect_line_module_import_spans(script);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            &script[spans[0].0 as usize..spans[0].1 as usize],
            "// FIXME: types\n// @ts-ignore\nimport Chart from \"chart.js/auto/auto\";"
        );
    }

    #[test]
    fn collect_import_span_leaves_regular_comments_in_script_body() {
        let script = "// import note\nimport Chart from \"chart.js/auto/auto\";\n";
        let spans = collect_line_module_import_spans(script);

        assert_eq!(spans.len(), 1);
        assert_eq!(
            &script[spans[0].0 as usize..spans[0].1 as usize],
            "import Chart from \"chart.js/auto/auto\";"
        );
    }
}
