use super::VirtualFile;
use crate::virtual_ts::{ProjectionFeatures, ProjectionMapping, VizeMapping};

/// Authenticate the one excess-property diagnostic loose mode may discard.
///
/// This is deliberately not a range ignore. The selected projection row must
/// be the canonical, editor-owned component props-literal entry, the checker
/// range must begin on that entry's quoted key, and the key's exact sub-span
/// must map to an authored attribute name inside a start tag. Inference-only
/// copies serve diagnostics only and therefore fail the `ALL` feature gate;
/// value/nested-object diagnostics do not begin in the key range.
///
/// In loose codegen every `$PropChecker` branch has an open record tail except
/// the direct `$IsLibraryGenericExport<C> extends true ? C` branch. A TS2353 on
/// this canonical key therefore proves that branch was selected. A false or
/// `true | false` result does not extend `true`, so non-library and union cases
/// retain the open tail and cannot reach this suppression.
pub(super) fn is_authored_component_literal_key(
    file: &VirtualFile,
    generated_start: u32,
    generated_end: u32,
    project: &crate::batch::VirtualProject,
) -> bool {
    let Some(source) = project.original_content_for_virtual(&file.virtual_path) else {
        return false;
    };
    let Some(mapping) = file.source_map.sfc_map.as_ref().map(|map| map.projection()) else {
        return false;
    };
    let generated_start = file
        .source_map
        .import_map
        .get_original_offset(generated_start) as usize;
    let generated_end = file
        .source_map
        .import_map
        .get_original_offset(generated_end) as usize;
    let restored = file.source_map.import_map.pre_rewrite_text(&file.content);
    let generated = restored.as_deref().unwrap_or(file.content.as_str());
    authenticated_component_key(
        generated,
        source,
        mapping,
        generated_start,
        generated_end.max(generated_start),
    )
}

fn authenticated_component_key(
    generated: &str,
    source: &str,
    mapping: &ProjectionMapping,
    generated_start: usize,
    generated_end: usize,
) -> bool {
    let Some(row) = narrowest_row(mapping, generated_start) else {
        return false;
    };
    if row.1 != ProjectionFeatures::ALL {
        return false;
    }
    let span = row.0;
    let Some(entry) = generated.get(span.gen_range.clone()) else {
        return false;
    };
    let Some(key_length) = quoted_key_length(entry) else {
        return false;
    };
    let key = span.gen_range.start..span.gen_range.start + key_length;
    if generated_start != key.start || generated_end > key.end {
        return false;
    }
    let Some(authored) = span
        .sub_spans
        .iter()
        .find(|sub_span| sub_span.gen_range == key)
        .map(|sub_span| {
            let base = mapping.authored_base();
            sub_span.src_range.start + base..sub_span.src_range.end + base
        })
    else {
        return false;
    };
    is_authored_attribute_key(source, authored)
}

fn narrowest_row(
    mapping: &ProjectionMapping,
    generated: usize,
) -> Option<(&VizeMapping, ProjectionFeatures)> {
    mapping
        .rows()
        .filter(|row| row.span.gen_range.contains(&generated))
        .min_by_key(|row| row.span.gen_range.len())
        .map(|row| (row.span, row.meta.features))
}

/// Length of an emitted quoted property key, including both quotes.
fn quoted_key_length(entry: &str) -> Option<usize> {
    let bytes = entry.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        match byte {
            b'\\' => escaped = true,
            b'"' if entry
                .get(index + 1..)
                .is_some_and(|tail| tail.starts_with(": ")) =>
            {
                return Some(index + 1);
            }
            _ => {}
        }
    }
    None
}

fn is_authored_attribute_key(source: &str, key: std::ops::Range<usize>) -> bool {
    let Some(token) = source.get(key.clone()) else {
        return false;
    };
    if token.is_empty()
        || token.bytes().any(|byte| {
            byte.is_ascii_whitespace() || matches!(byte, b'"' | b'\'' | b'<' | b'>' | b'=')
        })
    {
        return false;
    }
    let Some(before) = source.get(..key.start) else {
        return false;
    };
    let Some(tag_start) = before.rfind('<') else {
        return false;
    };
    if before.rfind('>').is_some_and(|end| end > tag_start) {
        return false;
    }
    let Some(tag_head) = source.get(tag_start + 1..key.start) else {
        return false;
    };
    if tag_head.starts_with(['/', '!', '?']) || !ends_outside_quotes(tag_head) {
        return false;
    }
    source.get(key.end..).is_some_and(|after| {
        after.is_empty() || after.starts_with(['=', '.', ' ', '\t', '\r', '\n', '/', '>'])
    })
}

fn ends_outside_quotes(tag_head: &str) -> bool {
    let mut quote = None;
    for byte in tag_head.bytes() {
        match quote {
            Some(open) if byte == open => quote = None,
            Some(_) => {}
            None if matches!(byte, b'"' | b'\'') => quote = Some(byte),
            None => {}
        }
    }
    quote.is_none()
}

#[cfg(test)]
mod tests {
    use super::authenticated_component_key;
    use crate::virtual_ts::{
        ProjectionFeatures, ProjectionMapping, ProjectionMeta, VizeMapping, VizeSubSpan,
    };

    fn projection(features: ProjectionFeatures) -> (String, ProjectionMapping, usize, usize) {
        let generated =
            String::from("check({\n  \"buffer\": 200,\n  \"nested\": { \"extra\": true },\n});");
        let buffer = generated.find("\"buffer\"").unwrap();
        let entry_end = generated[buffer..].find(',').unwrap() + buffer;
        let key_end = buffer + "\"buffer\"".len();
        let mut mapping = ProjectionMapping::new();
        mapping.push_with(
            VizeMapping {
                gen_range: buffer..entry_end,
                src_range: 17..30,
                sub_spans: vec![VizeSubSpan {
                    gen_range: buffer..key_end,
                    src_range: 18..24,
                }],
            },
            ProjectionMeta {
                features,
                ..ProjectionMeta::default()
            },
        );
        (generated, mapping, buffer, key_end)
    }

    #[test]
    fn accepts_only_the_editor_owned_component_attribute_key() {
        let source = "<template><Child :buffer=\"200\" /></template>";
        let (generated, mapping, start, end) = projection(ProjectionFeatures::ALL);
        assert!(authenticated_component_key(
            &generated, source, &mapping, start, end,
        ));
    }

    #[test]
    fn rejects_inference_copies_values_and_nested_object_keys() {
        let source = "<template><Child :buffer=\"200\" /></template>";
        let (generated, inference, start, end) = projection(ProjectionFeatures::DIAGNOSTICS);
        assert!(!authenticated_component_key(
            &generated, source, &inference, start, end,
        ));

        let (generated, primary, start, end) = projection(ProjectionFeatures::ALL);
        assert!(!authenticated_component_key(
            &generated,
            source,
            &primary,
            end + 2,
            end + 5,
        ));
        let nested = generated.find("\"extra\"").unwrap();
        assert!(!authenticated_component_key(
            &generated,
            source,
            &primary,
            nested,
            nested + "\"extra\"".len(),
        ));
        assert!(!authenticated_component_key(
            &generated,
            source,
            &primary,
            start,
            end + 1,
        ));
    }

    #[test]
    fn rejects_a_key_mapping_outside_a_component_start_tag() {
        let (generated, mapping, start, end) = projection(ProjectionFeatures::ALL);
        assert!(!authenticated_component_key(
            &generated,
            "const buffer = 200; //       ",
            &mapping,
            start,
            end,
        ));
    }
}
