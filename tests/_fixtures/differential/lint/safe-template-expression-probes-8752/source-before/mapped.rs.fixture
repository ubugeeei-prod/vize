//! Inferred probes for expressions Canon places in typed checks or handlers.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_canon::virtual_ts::VizeMapping;
use vize_l0::{String, cstr};

use super::TypeAwareDocument;

pub(super) fn bind_checked_expressions(document: &mut TypeAwareDocument, index: &mut u32) {
    let text = document.content.as_str();
    let mut edits = Vec::new();
    let mut allocator = Allocator::default();
    for row in document.mapping.rows() {
        let span = row.span;
        let candidates = std::iter::once((&span.gen_range, &span.src_range))
            .filter(|_| span.sub_spans.is_empty())
            .chain(
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
            } else if !line.contains("handler expression") {
                continue;
            }
            // The old reference wrapper already has its inferred probe.
            if line_start > 0 && super::binding_offset(text, generated.start as u32).is_some() {
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
            authored,
        ));
        updated.insert_str(at, &statement);
    }
    document.content = updated;
}
