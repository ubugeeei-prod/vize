//! Contextual component prop keys used by native hover.

use crate::batch::ImportSourceMap;

use super::{VizeMapping, VizeSemanticLink, VizeSemanticLinkKind};

/// Find the post-rewrite byte offset of the contextually typed object key that
/// corresponds to one authored component prop.
///
/// The ordinary semantic navigation key remains the authority for definition,
/// references and completion. Generic prop hover can query this repeated key
/// when that authority has `unknown` quick info: it sits inside the existing
/// checker call, where TypeScript has already instantiated the component's
/// generic signature from all authored values.
pub fn instantiated_component_prop_key_offset(
    code: &str,
    mappings: &[VizeMapping],
    semantic_links: &[VizeSemanticLink],
    import_source_map: &ImportSourceMap,
    source_offset: usize,
) -> Option<usize> {
    let mut identity = None;
    for link in semantic_links {
        if link.kind != VizeSemanticLinkKind::VueComponentPropNavigation {
            continue;
        }
        // Shared bridge documents retain semantic links in pre-rewrite
        // coordinates, while Maestro's owned document view normalizes those
        // links to the rewritten code. Authenticate either representation
        // against the producer mapping, which always remains pre-rewrite.
        for (generated, rewritten) in semantic_target_coordinates(import_source_map, link) {
            let Some(generated_name) = code.get(rewritten) else {
                continue;
            };
            for mapping in mappings {
                for (mapped, source) in spans(mapping) {
                    if *mapped != generated
                        || !(source.start <= source_offset && source_offset <= source.end)
                    {
                        continue;
                    }
                    let current = (source.clone(), generated_name);
                    if identity
                        .as_ref()
                        .is_some_and(|existing| *existing != current)
                    {
                        return None;
                    }
                    identity = Some(current);
                }
            }
        }
    }

    let (authored, generated_name) = identity?;
    let mut candidate = None;
    for mapping in mappings {
        for (generated, source) in spans(mapping) {
            if *source != authored {
                continue;
            }
            let generated = post_rewrite_range(import_source_map, generated);
            if code.get(generated.clone()) != Some(generated_name)
                || code.get(generated.start.saturating_sub(1)..generated.start) != Some("\"")
                || code.get(generated.end..generated.end.saturating_add(1)) != Some("\"")
            {
                continue;
            }
            if candidate.is_some_and(|existing| existing != generated.start) {
                return None;
            }
            candidate = Some(generated.start);
        }
    }
    candidate
}

fn spans(
    mapping: &VizeMapping,
) -> impl Iterator<Item = (&std::ops::Range<usize>, &std::ops::Range<usize>)> {
    std::iter::once((&mapping.gen_range, &mapping.src_range)).chain(
        mapping
            .sub_spans
            .iter()
            .map(|span| (&span.gen_range, &span.src_range)),
    )
}

fn post_rewrite_range(
    map: &ImportSourceMap,
    range: &std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    map.get_virtual_offset(range.start as u32) as usize
        ..map.get_virtual_offset(range.end as u32) as usize
}

fn pre_rewrite_range(
    map: &ImportSourceMap,
    range: &std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    map.get_original_offset(range.start as u32) as usize
        ..map.get_original_offset(range.end as u32) as usize
}

fn semantic_target_coordinates(
    map: &ImportSourceMap,
    link: &VizeSemanticLink,
) -> [(std::ops::Range<usize>, std::ops::Range<usize>); 2] {
    [
        (
            link.target_range.clone(),
            post_rewrite_range(map, &link.target_range),
        ),
        (
            pre_rewrite_range(map, &link.target_range),
            link.target_range.clone(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::OffsetAdjustment;

    #[test]
    fn accepts_pre_and_post_rewrite_semantic_link_coordinates() {
        let code = "0123456789___nav.items; ({\"items\": value});";
        let map = ImportSourceMap::new(vec![OffsetAdjustment {
            original_offset: 10,
            adjustment: 3,
        }]);
        let mappings = vec![
            VizeMapping {
                gen_range: 14..19,
                src_range: 5..10,
                sub_spans: Vec::new(),
            },
            VizeMapping {
                gen_range: 24..29,
                src_range: 5..10,
                sub_spans: Vec::new(),
            },
        ];

        for target_range in [14..19, 17..22] {
            let links = vec![VizeSemanticLink {
                source_range: 0..3,
                target_range,
                kind: VizeSemanticLinkKind::VueComponentPropNavigation,
            }];
            assert_eq!(
                instantiated_component_prop_key_offset(&code, &mappings, &links, &map, 7),
                Some(27),
            );
        }
    }
}
