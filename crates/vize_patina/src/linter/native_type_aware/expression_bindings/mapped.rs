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
            let indent_len = line.len() - trimmed.len();
            let indent = line.get(..indent_len).unwrap_or_default();
            let prefix = cstr!("{indent}const __expr_{index} = (");
            let statement = cstr!("{prefix}{expression});\n");
            edits.push((
                line_start,
                prefix.len(),
                expression.len(),
                authored.clone(),
                statement,
            ));
            *index += 1;
        }
    }
    edits.sort_unstable_by_key(|(at, _, _, _, _)| *at);
    // A mapped initializer can occur in more than one diagnostic row.
    edits.dedup_by(|left, right| left.0 == right.0 && left.3 == right.3);
    if edits.is_empty() {
        return;
    }
    let mut updated = String::from(text);
    for (at, prefix_len, expression_len, authored, statement) in edits.into_iter().rev() {
        document
            .mapping
            .note_generated_replacement(at, 0, statement.len());
        document.mapping.push(VizeMapping::new(
            at + prefix_len..at + prefix_len + expression_len,
            authored.clone(),
        ));
        mark_inferred_binding(document, at, &statement, authored);
        updated.insert_str(at, &statement);
    }
    document.content = updated;
}
