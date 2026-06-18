//! Module-scope facts collected from normal Vue `<script>` blocks.

use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Declaration, Statement};
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
    let mut offset = 0usize;
    for raw_line in script.split_inclusive('\n') {
        let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let trimmed = line.trim_start();
        if trimmed.starts_with("import ")
            || trimmed.starts_with("import\t")
            || (trimmed.starts_with("export ")
                && (trimmed.contains(" from ") || trimmed.starts_with("export type ")))
        {
            let start = offset + (line.len() - line.trim_start().len());
            let end = offset + line.len();
            spans.push((start as u32, end as u32));
        }
        offset += raw_line.len();
    }
    spans
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
