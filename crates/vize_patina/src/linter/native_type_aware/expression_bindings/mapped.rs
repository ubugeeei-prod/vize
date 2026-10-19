//! Inferred probes for expressions Canon places in typed checks or handlers.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_canon::virtual_ts::{ProjectionFeatures, ProjectionMeta, ProjectionSpanKind, VizeMapping};
use vize_l0::{String, cstr};

use super::TypeAwareDocument;

const INFERRED_BINDING: ProjectionMeta = ProjectionMeta {
    features: ProjectionFeatures::HOVER,
    kind: ProjectionSpanKind::TemplateExpression,
};

/// Only an owned inferred identifier can supply a whole-expression probe.
/// Its row moves with all later document edits, even for multiline values.
pub(super) fn inferred_binding_offset(
    document: &TypeAwareDocument,
    authored: &std::ops::Range<usize>,
) -> Option<u32> {
    document.mapping.rows().find_map(|row| {
        (row.meta == INFERRED_BINDING && &row.span.src_range == authored)
            .then(|| row.span.gen_range.end.checked_sub(1))
            .flatten()
            .and_then(|offset| u32::try_from(offset).ok())
    })
}

pub(super) fn mark_inferred_binding(
    document: &mut TypeAwareDocument,
    at: usize,
    statement: &str,
    authored: std::ops::Range<usize>,
) {
    let Some(prefix) = statement.find("const __expr_") else {
        return;
    };
    let start = prefix + "const ".len();
    let Some(end) = statement.get(start..).and_then(|rest| rest.find(" = ")) else {
        return;
    };
    let name = statement.get(start..start + end).unwrap_or_default();
    let end = name.find(':').unwrap_or(name.len());
    document.mapping.push_with(
        VizeMapping::new(at + start..at + start + end, authored),
        INFERRED_BINDING,
    );
}

pub(super) fn bind_checked_expressions(document: &mut TypeAwareDocument, index: &mut u32) {
    let text = document.content.as_str();
    let mut edits = Vec::new();
    let mut allocator = Allocator::default();
    for row in document.mapping.rows() {
        if row.meta == INFERRED_BINDING {
            continue;
        }
        let span = row.span;
        let candidates = std::iter::once((&span.gen_range, &span.src_range)).chain(
            span.sub_spans
                .iter()
                .map(|sub| (&sub.gen_range, &sub.src_range)),
        );
        for (generated, authored) in candidates {
            let Some(before) = text.get(..generated.start) else {
                continue;
            };
            let line_start = before.rfind('\n').map_or(0, |at| at + 1);
            let line_end = text
                .get(generated.end..)
                .and_then(|rest| rest.find('\n'))
                .map_or(text.len(), |at| generated.end + at);
            let Some(line) = text.get(line_start..line_end) else {
                continue;
            };
            let trimmed = line.trim_start();
            // A complete first condition keeps its lexical and narrowing scope.
            // An else-if or synthetic enclosing guard needs a different owner;
            // inserting a declaration before either would alter control flow.
            let condition = text
                .get(line_start..generated.start)
                .and_then(|prefix| prefix.trim_start().strip_prefix("if ("))
                .is_some_and(|wrapper| {
                    if !wrapper.bytes().all(|byte| byte == b'(') {
                        return false;
                    }
                    let suffix = text.get(generated.end..).unwrap_or_default();
                    let closing = suffix.bytes().take_while(|byte| *byte == b')').count();
                    closing == wrapper.len() + 1
                        && suffix
                            .get(closing..)
                            .is_some_and(|tail| tail.starts_with(" {\n"))
                });
            let checked = [
                "const __vize_prop_check_",
                "const __vize_native_prop_check_",
                "const __vize_handler_",
            ]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix));
            if checked {
                // The identifier and wrapper anchor are synthetic; only the
                // mapped initializer owns an expression type.
                let Some(assign) = line.find(" = ") else {
                    continue;
                };
                if generated.start < line_start + assign + 3 {
                    continue;
                }
                // A token inside an initializer does not own the initializer's
                // inferred type. Only its complete mapped value is eligible.
                let prefix = text
                    .get(line_start + assign + 3..generated.start)
                    .unwrap_or_default()
                    .trim();
                let suffix = text.get(generated.end..line_end).unwrap_or_default().trim();
                if !prefix.chars().all(|character| character == '(')
                    || !suffix.trim_start_matches(')').trim_start().starts_with(';')
                {
                    continue;
                }
            } else if !condition
                && !guard_header(trimmed)
                && !line.trim_end().ends_with(";  // handler expression")
                && !line
                    .trim_end()
                    .ends_with(";  // handler expression (emit checks disabled)")
            {
                continue;
            }
            // The old reference wrapper already has its inferred probe.
            if inferred_binding_offset(document, authored).is_some() {
                continue;
            }
            if generated.len() != authored.len() {
                continue;
            }
            let Some(expression) = text.get(generated.clone()) else {
                continue;
            };
            let complete = Parser::new(&allocator, expression, SourceType::ts())
                .parse_expression()
                .is_ok_and(|parsed| parsed.span().end as usize == expression.trim_end().len());
            allocator.reset();
            if !complete {
                continue;
            }
            let name = cstr!("__expr_{index}");
            *index += 1;
            if condition || checked || !guard_header(trimmed) {
                let indent_len = line.len() - trimmed.len();
                let indent = line.get(..indent_len).unwrap_or_default();
                let prefix = cstr!("{indent}const {name} = (");
                let name_start = indent_len + "const ".len();
                edits.push(Edit {
                    at: line_start,
                    text: cstr!("{prefix}{expression});\n"),
                    authored: authored.clone(),
                    probe: Some(Probe {
                        name: name_start..name_start + name.len(),
                        copy: prefix.len()..prefix.len() + expression.len(),
                    }),
                    inline: false,
                });
                continue;
            }
            // A nested or else-if condition evaluates after other guard terms,
            // where no declaration can sit. An IIFE parameter is inferred from
            // its argument in place; the comma keeps narrowing on the original.
            let prefix = cstr!("((({name}) => {name})(");
            edits.push(Edit {
                at: generated.start,
                text: cstr!("{prefix}{expression}), "),
                authored: authored.clone(),
                probe: Some(Probe {
                    name: 3..3 + name.len(),
                    copy: prefix.len()..prefix.len() + expression.len(),
                }),
                inline: true,
            });
            edits.push(Edit {
                at: generated.end,
                text: String::from(")"),
                authored: authored.clone(),
                probe: None,
                inline: true,
            });
        }
    }
    edits.sort_by_key(|edit| edit.at);
    // A mapped initializer can occur in more than one diagnostic row. A
    // declared probe is preferred; otherwise the first inline copy owns it.
    let declared: Vec<_> = edits
        .iter()
        .filter(|edit| !edit.inline)
        .map(|edit| edit.authored.clone())
        .collect();
    let mut kept = Vec::new();
    edits.retain(|edit| {
        if edit.inline && declared.contains(&edit.authored) {
            return false;
        }
        let at = (!edit.inline).then_some(edit.at);
        let key = (at, edit.probe.is_some(), edit.authored.clone());
        if kept.contains(&key) {
            return false;
        }
        kept.push(key);
        true
    });
    if edits.is_empty() {
        return;
    }
    let mut updated = String::from(text);
    for edit in edits.into_iter().rev() {
        document
            .mapping
            .note_generated_replacement(edit.at, 0, edit.text.len());
        if let Some(Probe { name, copy }) = edit.probe {
            document.mapping.push(VizeMapping::new(
                edit.at + copy.start..edit.at + copy.end,
                edit.authored.clone(),
            ));
            document.mapping.push_with(
                VizeMapping::new(edit.at + name.start..edit.at + name.end, edit.authored),
                INFERRED_BINDING,
            );
        }
        updated.insert_str(edit.at, &edit.text);
    }
    document.content = updated;
}

/// Generated text inserted for one authored expression.
struct Edit {
    at: usize,
    text: String,
    authored: std::ops::Range<usize>,
    /// The probe name and expression copy, as offsets within `text`.
    probe: Option<Probe>,
    inline: bool,
}

struct Probe {
    name: std::ops::Range<usize>,
    copy: std::ops::Range<usize>,
}

/// An `if` / `else if` header line.
fn guard_header(trimmed: &str) -> bool {
    let header = trimmed.strip_prefix('}').unwrap_or(trimmed).trim_start();
    (header.starts_with("if (") || header.starts_with("else if ("))
        && trimmed.trim_end().ends_with(") {")
}
