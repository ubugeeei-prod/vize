use super::VirtualFile;
use crate::batch::component_literal_key::authenticate_component_literal_key;
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
/// In loose codegen every `$PropChecker` branch has an open record tail. The
/// library branch gains it from `C | ((props: T) => void)` when the declaration
/// predates `__VLS_PROPS_FALLBACK` (#13021), so the checker itself no longer
/// reports this TS2353 on any host. This filter stays as the backstop shared
/// with the negotiated content-mapper selector; it never widens what loose
/// mode accepts beyond an authenticated authored attribute key.
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
    authenticate_component_literal_key(source, generated, row.0, row.1, mapping.authored_base())
        .is_some_and(|key| {
            generated_start == key.generated.start && generated_end <= key.generated.end
        })
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
