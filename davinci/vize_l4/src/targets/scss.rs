//! Expanded CSS for a complete original SCSS program.
use crate::write::EmitDocument;
use vize_l0::{String, line_index::LineBreaks};
use vize_l2::lang::scss::ScssProgram;

pub fn emit(program: &ScssProgram<'_>) -> String {
    let mut css = EmitDocument::new(false);
    write::<false>(&mut css, program);
    css.into_string()
}

/// Append one expanded stylesheet directly to its final CSS document. Dart
/// Sass maps each selector and property name, plus a value if its origin
/// starts on a different source line than the property.
pub fn append(css: &mut EmitDocument, program: &ScssProgram<'_>) {
    reserve_style(css, program);
    if css.is_recording() {
        write::<true>(css, program);
    } else {
        write::<false>(css, program);
    }
}

fn reserve_style(css: &mut EmitDocument, program: &ScssProgram<'_>) {
    // Reserve a complete style at its join point. A stream of small writes
    // otherwise grows the final buffer past the capacity of the fragment join.
    let mut bytes = 0;
    let mut group = None;
    for rule in program.runs() {
        if let Some(previous) = group {
            bytes += 1 + usize::from(previous != rule.group);
        }
        bytes += rule.selector.len() + " {\n}".len();
        for property in &rule.declarations {
            bytes +=
                "  ".len() + property.name.len() + ": ".len() + property.value.len() + ";\n".len();
        }
        group = Some(rule.group);
    }
    css.reserve_text(bytes);
}

fn write<const MAPPED: bool>(css: &mut EmitDocument, program: &ScssProgram<'_>) {
    let block = program.block();
    let mut line_start = block.start();
    let mut lines = LineBreaks::Lsp
        .line_starts(block.source())
        .skip(1)
        .map(|at| block.start() + at as u32)
        .peekable();
    let mut previous_group = None;
    for rule in program.runs() {
        if let Some(previous) = previous_group {
            css.push_char('\n');
            if previous != rule.group {
                css.push_char('\n');
            }
        }
        if MAPPED {
            css.push_linked(&rule.selector, rule.selector_span);
        } else {
            css.push_str(&rule.selector);
        }
        css.push_str(" {\n");
        for property in &rule.declarations {
            css.push_str("  ");
            if MAPPED {
                css.push_linked(property.name, property.name_span);
            } else {
                css.push_str(property.name);
            }
            css.push_str(": ");
            // Declarations are emitted in authored order. Walk line breaks
            // once even when many properties refer back to one earlier value.
            let origin = if MAPPED {
                while let Some(start) = lines.next_if(|start| *start <= property.name_span.start) {
                    line_start = start;
                }
                let end = lines.peek().copied().unwrap_or_else(|| block.end());
                program
                    .value_origin(property.value_span)
                    .filter(|span| !(line_start..end).contains(&span.start))
            } else {
                None
            };
            if let Some(span) = origin {
                css.push_linked(&property.value, span);
            } else {
                css.push_str(&property.value);
            }
            css.push_str(";\n");
        }
        css.push_char('}');
        previous_group = Some(rule.group);
    }
}
