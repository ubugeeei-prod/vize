//! Template diagnostic directives for the TypeScript content mapper.
//!
//! The native checker and Content Mapper use the same authored-node ownership.
//! Protocol v1 can encode multiple `@vue-ignore` / `@vue-skip` intervals, but
//! expects must retain one interval until TypeScript supports aggregating
//! usage across intervals of the same authored expectation.

use std::ops::Range;

use vize_atelier_sfc::SfcDescriptor;
use vize_l0::String as CompactString;

use super::super::vue_codegen::generic_attribute::GenericCopies;
use crate::template_diagnostic_directives::{DirectivePolicy, TemplateDiagnosticDirectives};

use super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperSpan,
    ContentMapperUnusedExpectDiagnostic, DIRECTIVE_POLICY_EXPECT, DIRECTIVE_POLICY_IGNORE,
};
use crate::virtual_ts::{NAMED_PROP_DIAGNOSTIC_MARKER, VizeMapping};

/// Mapper diagnostic code for an unused `@vue-expect-error` directive.
const UNUSED_EXPECT_CODE: i32 = 4;

/// Project comments onto checks for their authored target nodes. Returns
/// `None` when the SFC declares no directives that reach the mapper.
pub(super) fn sfc_diagnostic_directives(
    source: &str,
    descriptor: &SfcDescriptor<'_>,
    spans: &[ContentMapperSpan],
) -> Option<ContentMapperDiagnosticDirectives> {
    let authored = TemplateDiagnosticDirectives::for_descriptor(source, descriptor);
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

/// Generated copies of the `generic` attribute repeat the diagnostics of the
/// one mapped copy; ignore them so each authored error reports once.
pub(super) fn with_generic_echoes(
    directives: Option<ContentMapperDiagnosticDirectives>,
    copies: Option<&GenericCopies>,
) -> Option<ContentMapperDiagnosticDirectives> {
    let Some(copies) = copies else {
        return directives;
    };
    let mut directives = directives.unwrap_or(ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics: Vec::new(),
        directives: Vec::new(),
    });
    directives.directives.extend(copies.echoes().map(|echo| {
        ContentMapperDiagnosticDirective([
            copies.attribute.start,
            copies.attribute.len(),
            echo.start,
            echo.end,
            DIRECTIVE_POLICY_IGNORE,
            0,
        ])
    }));
    Some(directives)
}

/// Suppress the per-prop assignment when the canonical whole-props literal
/// contains the same authored binding. Native Vize collapses those two mapped
/// diagnostics after checking; the upstream content-mapper host reports every
/// virtual diagnostic independently, so the earlier assignment is an echo.
///
/// Keep the assignment when the literal deliberately carries `never` for an
/// inline callback, or when the literal entry is itself `@ts-ignore`d before a
/// spread: in both cases the assignment is the only diagnostic owner.
pub(super) fn with_component_prop_echoes(
    directives: Option<ContentMapperDiagnosticDirectives>,
    generated: &str,
    mappings: &[VizeMapping],
) -> Option<ContentMapperDiagnosticDirectives> {
    let literal_owners = mappings
        .iter()
        .filter(|candidate| {
            generated
                .get(candidate.gen_range.clone())
                .is_some_and(|entry| {
                    entry.starts_with('"')
                        && entry.contains("\": ")
                        && !entry.contains("undefined as never")
                        && !preceded_by_named_prop_ignore(generated, candidate.gen_range.start)
                })
        })
        .map(|candidate| &candidate.src_range)
        .collect::<Vec<_>>();
    let mut echoes = Vec::new();
    for mapping in mappings {
        let Some(text) = generated.get(mapping.gen_range.clone()) else {
            continue;
        };
        if !text.trim_start().starts_with("const __vize_prop_check_") {
            continue;
        }
        let has_literal_owner = literal_owners
            .iter()
            .any(|source| **source == mapping.src_range);
        if has_literal_owner {
            echoes.push(ContentMapperDiagnosticDirective([
                mapping.src_range.start,
                mapping.src_range.len(),
                mapping.gen_range.start,
                mapping.gen_range.end,
                DIRECTIVE_POLICY_IGNORE,
                0,
            ]));
        }
    }
    if echoes.is_empty() {
        return directives;
    }
    let mut directives = directives.unwrap_or(ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics: Vec::new(),
        directives: Vec::new(),
    });
    for echo in echoes {
        let [_, _, start, end, _, _] = echo.0;
        let overlaps_existing = directives.directives.iter().any(|existing| {
            let [_, _, existing_start, existing_end, _, _] = existing.0;
            start < existing_end && existing_start < end
        });
        if !overlaps_existing {
            directives.directives.push(echo);
        }
    }
    directives
        .directives
        .sort_unstable_by_key(|directive| directive.0[2]);
    Some(directives)
}

fn preceded_by_named_prop_ignore(generated: &str, start: usize) -> bool {
    let Some(line_start) = generated.get(..start).and_then(|prefix| prefix.rfind('\n')) else {
        return false;
    };
    generated
        .get(..line_start)
        .and_then(|prefix| prefix.rsplit_once('\n').map(|(_, line)| line))
        .is_some_and(|line| line.contains(NAMED_PROP_DIAGNOSTIC_MARKER))
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
