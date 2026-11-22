//! Expanded CSS for a complete original SCSS program.
use crate::write::EmitDocument;
use vize_l0::String;
use vize_l2::lang::scss::ScssProgram;

pub fn emit(program: &ScssProgram<'_>) -> String {
    let mut css = EmitDocument::new(false);
    append(&mut css, program);
    css.into_string()
}

/// Append one expanded stylesheet directly to its final CSS document. Dart
/// Sass maps each emitted selector and property name to its own authored node.
pub fn append(css: &mut EmitDocument, program: &ScssProgram<'_>) {
    let mut previous_group = None;
    for rule in program.runs() {
        if let Some(previous) = previous_group {
            css.push_char('\n');
            if previous != rule.group {
                css.push_char('\n');
            }
        }
        css.push_linked(&rule.selector, rule.selector_span);
        css.push_str(" {\n");
        for property in &rule.declarations {
            css.push_str("  ");
            css.push_linked(property.name, property.name_span);
            css.push_str(": ");
            css.push_str(&property.value);
            css.push_str(";\n");
        }
        css.push_char('}');
        previous_group = Some(rule.group);
    }
}
