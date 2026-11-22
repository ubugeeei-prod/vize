//! Expanded CSS for a complete original SCSS program.
#[cfg(feature = "scoped-scss")]
use crate::module::ScopeId;
use crate::write::EmitDocument;
use vize_l0::{String, line_index::LineBreaks};
use vize_l2::lang::scss::ScssProgram;

pub fn emit(program: &ScssProgram<'_>) -> String {
    let mut css = EmitDocument::new(false);
    write::<false>(
        &mut css,
        program,
        #[cfg(feature = "scoped-scss")]
        None,
    );
    css.into_string()
}

/// Append one expanded stylesheet directly to its final CSS document. Dart
/// Sass maps each selector and property name, plus a value if its origin
/// starts on a different source line than the property.
pub fn append(css: &mut EmitDocument, program: &ScssProgram<'_>) {
    reserve_style(
        css,
        program,
        #[cfg(feature = "scoped-scss")]
        None,
    );
    if css.is_recording() {
        write::<true>(
            css,
            program,
            #[cfg(feature = "scoped-scss")]
            None,
        );
    } else {
        write::<false>(
            css,
            program,
            #[cfg(feature = "scoped-scss")]
            None,
        );
    }
}

/// Scope the rightmost compound of every selector in the admitted rule program.
#[cfg(feature = "scoped-scss")]
pub fn append_scoped(
    css: &mut EmitDocument,
    program: &ScssProgram<'_>,
    id: ScopeId<'_>,
    trim: bool,
) {
    let scope = Some((id, trim));
    reserve_style(css, program, scope);
    if css.is_recording() {
        write::<true>(css, program, scope);
    } else {
        write::<false>(css, program, scope);
    }
}

fn reserve_style(
    css: &mut EmitDocument,
    program: &ScssProgram<'_>,
    #[cfg(feature = "scoped-scss")] scope: Option<(ScopeId<'_>, bool)>,
) {
    // Reserve a complete style at its join point. A stream of small writes
    // otherwise grows the final buffer past the capacity of the fragment join.
    let mut bytes = 0;
    let mut group = None;
    for rule in program.runs() {
        if let Some(previous) = group {
            let gap = previous != rule.group;
            #[cfg(feature = "scoped-scss")]
            let gap = gap && !scope.is_some_and(|(_, trim)| trim);
            bytes += 1 + usize::from(gap);
        }
        #[cfg(feature = "scoped-scss")]
        if let Some((id, _)) = scope {
            bytes += id.as_str().len() + 2;
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

fn write<const MAPPED: bool>(
    css: &mut EmitDocument,
    program: &ScssProgram<'_>,
    #[cfg(feature = "scoped-scss")] scope: Option<(ScopeId<'_>, bool)>,
) {
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
            let gap = previous != rule.group;
            #[cfg(feature = "scoped-scss")]
            let gap = gap && !scope.is_some_and(|(_, trim)| trim);
            if gap {
                css.push_char('\n');
            }
        }
        if MAPPED {
            css.push_linked(&rule.selector, rule.selector_span);
        } else {
            css.push_str(&rule.selector);
        }
        #[cfg(feature = "scoped-scss")]
        if let Some((id, _)) = scope {
            // L2 admits only type/class compounds, descendants and expanded Sass
            // parent selectors. None can end in a pseudo, combinator or list.
            css.push_char('[');
            css.push_str(id.as_str());
            css.push_char(']');
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
