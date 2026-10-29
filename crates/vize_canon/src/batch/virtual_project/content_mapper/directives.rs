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
use crate::virtual_ts::VizeMapping;

/// Mapper diagnostic code for an unused `@vue-expect-error` directive.
const UNUSED_EXPECT_CODE: i32 = 4;

/// Project comments onto checks for their authored target nodes. Returns
/// `None` when the SFC declares no directives that reach the mapper.
pub(super) fn sfc_diagnostic_directives(
    source: &str,
    generated: &str,
    descriptor: &SfcDescriptor<'_>,
    spans: &[ContentMapperSpan],
    echoes: &[crate::virtual_ts::diagnostic_ownership::DiagnosticEcho],
) -> Option<ContentMapperDiagnosticDirectives> {
    let authored = TemplateDiagnosticDirectives::for_descriptor(source, descriptor);
    let mut directives = Vec::new();
    let mut expects_unused_table = false;
    let owned_echoes = echoes
        .iter()
        .map(|echo| &echo.generated)
        .collect::<Vec<_>>();

    for directive in authored.directives() {
        // Exported navigation mappings stay untouched. Only emitter-proven
        // echoes with a real primary are excluded from EXPECT ownership.
        let excluded = if directive.policy == DirectivePolicy::Expect {
            owned_echoes.as_slice()
        } else {
            &[]
        };
        let ranges = mapped_virtual_ranges(spans, &directive.targets, excluded);
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
            // when only one of its ranges has a diagnostic. Navigation reads
            // carry no diagnostic, so they must not stretch that one interval
            // across a later sibling.
            let (start, end) = expect_interval(generated, spans, &directive.targets, &ranges);
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
/// inline callback, or when the literal entry is itself `@ts-ignore`d (before
/// a spread, or as an inference-only callback copy whose context may be `any`):
/// in each case the assignment is the only diagnostic owner (#10739).
pub(super) fn with_component_prop_echoes(
    directives: Option<ContentMapperDiagnosticDirectives>,
    generated: &str,
    mappings: &[VizeMapping],
    callback_primaries: &[Range<usize>],
) -> Option<ContentMapperDiagnosticDirectives> {
    // A linear index of actual emitted statement identities. Text equality of
    // a callback in another invocation cannot establish its contextual type.
    let protected: vize_l0::FxHashSet<_> =
        callback_primaries.iter().map(|range| range.start).collect();
    let literal_owners = mappings
        .iter()
        .filter(|candidate| {
            generated
                .get(candidate.gen_range.clone())
                .is_some_and(|entry| {
                    entry.starts_with('"')
                        && entry.contains("\": ")
                        && !entry.contains("undefined as never")
                        && !preceded_by_ts_ignore(generated, candidate.gen_range.start)
                        && !preceded_by_named_prop_ignore(generated, candidate.gen_range.start)
                })
        })
        .map(|candidate| &candidate.src_range)
        .collect::<Vec<_>>();
    let mut echoes = Vec::new();
    for mapping in mappings {
        if protected.contains(&mapping.gen_range.start) {
            continue;
        }
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

/// Only an emitter-proven repeated expression is excluded. Put synthetic
/// ignores first: upstream marks the first matching tuple used, so an echo
/// must never satisfy an authored expect directive on a different check.
pub(super) fn with_owned_input_echoes(
    directives: Option<ContentMapperDiagnosticDirectives>,
    echoes: &[crate::virtual_ts::diagnostic_ownership::DiagnosticEcho],
) -> Option<ContentMapperDiagnosticDirectives> {
    let mut covered_ignores = directives
        .as_ref()
        .into_iter()
        .flat_map(|table| &table.directives)
        .filter_map(|directive| {
            let [_, _, start, end, policy, _] = directive.0;
            (policy == DIRECTIVE_POLICY_IGNORE && start < end).then_some(start..end)
        })
        .collect::<Vec<_>>();
    covered_ignores.sort_unstable_by_key(|range| range.start);
    let mut union: Vec<Range<usize>> = Vec::with_capacity(covered_ignores.len());
    for range in covered_ignores {
        if let Some(last) = union.last_mut()
            && range.start <= last.end
        {
            last.end = last.end.max(range.end);
        } else {
            union.push(range);
        }
    }
    let mut owned = Vec::new();
    for echo in echoes
        .iter()
        .filter(|echo| echo.suppress && !echo.authored.is_empty())
    {
        let mut start = echo.generated.start;
        let first = union.partition_point(|range| range.end <= start);
        for covered in union.iter().skip(first) {
            if covered.start >= echo.generated.end {
                break;
            }
            if start < covered.start {
                owned.push(ContentMapperDiagnosticDirective([
                    echo.authored.start,
                    echo.authored.len(),
                    start,
                    covered.start,
                    DIRECTIVE_POLICY_IGNORE,
                    0,
                ]));
            }
            start = start.max(covered.end);
        }
        if start < echo.generated.end {
            owned.push(ContentMapperDiagnosticDirective([
                echo.authored.start,
                echo.authored.len(),
                start,
                echo.generated.end,
                DIRECTIVE_POLICY_IGNORE,
                0,
            ]));
        }
    }
    if owned.is_empty() {
        return directives;
    }
    let mut directives = directives.unwrap_or(ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics: Vec::new(),
        directives: Vec::new(),
    });
    owned.append(&mut directives.directives);
    directives.directives = owned;
    Some(directives)
}

fn preceded_by_ts_ignore(generated: &str, start: usize) -> bool {
    let Some(line_start) = generated.get(..start).and_then(|prefix| prefix.rfind('\n')) else {
        return false;
    };
    generated
        .get(..line_start)
        .and_then(|prefix| prefix.rsplit_once('\n').map(|(_, line)| line))
        .is_some_and(|line| line.trim_start().starts_with("// @ts-ignore"))
}

fn expect_interval(
    generated: &str,
    spans: &[ContentMapperSpan],
    targets: &[Range<usize>],
    ranges: &[Range<usize>],
) -> (usize, usize) {
    let Some((start, end)) = range_bounds(ranges) else {
        return (0, 0);
    };
    let cuts = foreign_diagnostic_cuts(generated, spans, targets, start, end);
    if cuts.is_empty() {
        return (start, end);
    }
    let kept = ranges
        .iter()
        .filter(|range| !range_contains_cut(range, &cuts))
        .cloned()
        .collect::<Vec<_>>();
    let clusters = clusters_split_by_cuts(&kept, &cuts);
    clusters
        .into_iter()
        .max_by_key(|cluster| {
            let text = generated.get(cluster.start..cluster.end).unwrap_or("");
            let rank = if text.contains("__vize_prop_check_") {
                3_u8
            } else if text.contains("__vize_event_keys_") || text.contains("__vize_required_slots_")
            {
                2
            } else {
                1
            };
            (rank, cluster.end - cluster.start)
        })
        .map_or((0, 0), |cluster| (cluster.start, cluster.end))
}

fn range_bounds(ranges: &[Range<usize>]) -> Option<(usize, usize)> {
    Some((ranges.first()?.start, ranges.last()?.end))
}

fn range_contains_cut(range: &Range<usize>, cuts: &[usize]) -> bool {
    cuts.iter()
        .any(|cut| range.start < *cut && *cut < range.end)
}

fn clusters_split_by_cuts(ranges: &[Range<usize>], cuts: &[usize]) -> Vec<Range<usize>> {
    let mut clusters = Vec::new();
    for range in ranges {
        let separated = clusters.last().is_some_and(|last: &Range<usize>| {
            cuts.iter()
                .any(|cut| last.end <= *cut && *cut < range.start)
        });
        if separated || clusters.is_empty() {
            clusters.push(range.clone());
            continue;
        }
        if let Some(last) = clusters.last_mut() {
            last.end = last.end.max(range.end);
        }
    }
    clusters
}

fn foreign_diagnostic_cuts(
    generated: &str,
    spans: &[ContentMapperSpan],
    targets: &[Range<usize>],
    start: usize,
    end: usize,
) -> Vec<usize> {
    let mut cuts = spans
        .iter()
        .filter_map(|span| {
            let ContentMapperSpan([generated_at, length, original, original_length, _, _]) = *span;
            if length == 0 || generated_at < start || end <= generated_at {
                return None;
            }
            let overlaps_target = targets
                .iter()
                .any(|target| original < target.end && original + original_length > target.start);
            (!overlaps_target && template_expression_cut(generated, generated_at))
                .then_some(generated_at)
        })
        .collect::<Vec<_>>();
    cuts.sort_unstable();
    cuts.dedup();
    cuts
}

fn template_expression_cut(generated: &str, start: usize) -> bool {
    let line_start = generated
        .get(..start)
        .and_then(|before| before.rfind('\n'))
        .map_or(0, |index| index + 1);
    let Some(rest) = generated.get(line_start..) else {
        return false;
    };
    let line = rest
        .split_once('\n')
        .map_or(rest, |(line, _)| line)
        .trim_start();
    line.starts_with("void (") && !line.contains("__vize_")
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
    excluded: &[&Range<usize>],
) -> Vec<Range<usize>> {
    let mut ranges = spans
        .iter()
        .filter_map(
            |ContentMapperSpan([generated, length, original, original_length, _, _])| {
                let echo = excluded.partition_point(|echo| echo.end <= *generated);
                let is_owned_echo = excluded
                    .get(echo)
                    .is_some_and(|echo| echo.start <= *generated && generated + length <= echo.end);
                (*length > 0
                    && !is_owned_echo
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
