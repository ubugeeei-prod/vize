//! Template diagnostic directives for the TypeScript content mapper.
//!
//! The native checker and Content Mapper use the same authored-node ownership.
//! Protocol v1 can encode multiple `@vue-ignore` / `@vue-skip` intervals, but
//! expects must retain one interval until TypeScript supports aggregating
//! usage across intervals of the same authored expectation.

use std::ops::Range;

use vize_atelier_sfc::SfcTemplateBlock;
use vize_l0::String as CompactString;

use crate::template_diagnostic_directives::{DirectivePolicy, TemplateDiagnosticDirectives};

use super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperSpan,
    ContentMapperUnusedExpectDiagnostic, DIRECTIVE_POLICY_EXPECT, DIRECTIVE_POLICY_IGNORE,
};

/// Mapper diagnostic code for an unused `@vue-expect-error` directive.
const UNUSED_EXPECT_CODE: i32 = 4;

/// Project comments onto checks for their authored target nodes. Returns
/// `None` when the template declares no directives that reach the mapper.
pub(super) fn template_diagnostic_directives(
    template: Option<&SfcTemplateBlock<'_>>,
    spans: &[ContentMapperSpan],
) -> Option<ContentMapperDiagnosticDirectives> {
    let template = template?;
    let authored = TemplateDiagnosticDirectives::for_template(template);
    let mut directives = Vec::new();
    let mut expects_unused_table = false;

    for directive in authored.directives() {
        let ranges = mapped_virtual_ranges(spans, &directive.targets);
        let policy = match directive.policy {
            DirectivePolicy::Expect => {
                expects_unused_table = true;
                DIRECTIVE_POLICY_EXPECT
            }
            DirectivePolicy::Ignore | DirectivePolicy::Skip => DIRECTIVE_POLICY_IGNORE,
        };

        if policy == DIRECTIVE_POLICY_EXPECT {
            // TypeScript v1 marks each tuple used independently. Sending one
            // tuple per disjoint range would emit a false unused expectation
            // when only one of its ranges has a diagnostic.
            let (start, end) = match (ranges.first(), ranges.last()) {
                (Some(first), Some(last)) => (first.start, last.end),
                _ => (0, 0),
            };
            directives.push(ContentMapperDiagnosticDirective([
                directive.token.start,
                directive.token.len(),
                start,
                end,
                policy,
                0,
            ]));
        } else {
            directives.extend(ranges.into_iter().map(|range| {
                ContentMapperDiagnosticDirective([
                    directive.token.start,
                    directive.token.len(),
                    range.start,
                    range.end,
                    policy,
                    0,
                ])
            }));
        }
    }

    if directives.is_empty() {
        return None;
    }
    let unused_expect_directive_diagnostics = if expects_unused_table {
        vec![ContentMapperUnusedExpectDiagnostic {
            code: UNUSED_EXPECT_CODE,
            message_text: CompactString::from("Unused '@vue-expect-error' directive"),
        }]
    } else {
        Vec::new()
    };
    Some(ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics,
        directives,
    })
}

fn mapped_virtual_ranges(
    spans: &[ContentMapperSpan],
    targets: &[Range<usize>],
) -> Vec<Range<usize>> {
    let mut ranges = spans
        .iter()
        .filter_map(
            |ContentMapperSpan([generated, length, original, original_length, _, _])| {
                (*length > 0
                    && targets.iter().any(|target| {
                        *original < target.end && original + original_length > target.start
                    }))
                .then_some(*generated..generated + length)
            },
        )
        .collect::<Vec<_>>();
    ranges.sort_unstable_by_key(|range| range.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
            continue;
        }
        merged.push(range);
    }
    merged
}
