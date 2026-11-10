//! Publish generated code with both binding metadata carriers.

use super::super::{
    TextBuf, VirtualTsGenerationOptions, VirtualTsOutput, VizeMapping, VizeSemanticLink,
};
use vize_carton::String;

pub(super) fn finish(ts: TextBuf, options: VirtualTsGenerationOptions<'_>) -> vize_carton::String {
    if ts.contains(super::super::ATTR_VALUE_REFERENCE)
        && let Some(collector) = options.diagnostic_ownership
    {
        let mut helper = TextBuf::default();
        helper.push_lib_text(super::super::attr_value_declaration(
            options.hoist_shared_preamble,
        ));
        collector
            .borrow_mut()
            .record_attr_value_helper(helper.into_compact());
    }
    ts.into_compact()
}

pub(super) fn publish(
    ts: String,
    mappings: Vec<VizeMapping>,
    diagnostic_mappings: Vec<VizeMapping>,
    semantic_links: Vec<VizeSemanticLink>,
    prop_default_key_links: Vec<VizeSemanticLink>,
    component_export: Option<std::ops::Range<usize>>,
) -> VirtualTsOutput {
    let mut mapping =
        super::super::mapping::ProjectionMapping::from_parts(mappings, semantic_links);
    for span in diagnostic_mappings {
        mapping.push_with(
            span,
            super::super::mapping::ProjectionMeta {
                features: super::super::mapping::ProjectionFeatures::DIAGNOSTICS,
                ..Default::default()
            },
        );
    }
    // The exported name's authored counterpart is the SFC itself.
    if let Some(name) = component_export {
        mapping.push_with(
            VizeMapping::new(name, 0..0),
            super::super::mapping::ProjectionMeta {
                features: super::super::mapping::ProjectionFeatures::DEFINITION
                    .union(super::super::mapping::ProjectionFeatures::REFERENCES),
                kind: super::super::mapping::ProjectionSpanKind::ComponentExport,
            },
        );
    }
    mapping.set_prop_default_key_links(prop_default_key_links);
    super::super::mapping::publish_virtual_ts(VirtualTsOutput { code: ts, mapping })
}

#[cfg(test)]
mod tests {
    use super::publish;
    use crate::virtual_ts::VizeMapping;
    use crate::virtual_ts::mapping::{ProjectionFeatures, ProjectionMeta, ProjectionSpanKind};
    use vize_carton::String;

    #[test]
    fn pinned_sfc_export_only_advertises_navigation() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/component-export-navigation-features/Probe.vue.txt"
        ));
        let descriptor =
            vize_atelier_sfc::parse_sfc(source, vize_atelier_sfc::SfcParseOptions::default())
                .expect("pinned SFC");
        let template = descriptor.template.as_ref().expect("template");
        let script = descriptor.script_setup.as_ref().expect("setup script");
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, &template.content);
        assert_eq!(errors.len(), 0);
        let summary = vize_atelier_sfc::croquis::analyze_sfc_descriptor(
            &descriptor,
            Some(&root),
            vize_atelier_sfc::croquis::SfcCroquisOptions::full(),
        );
        let output = super::super::generate_virtual_ts_with_offsets_and_checks(
            &summary,
            Some(&script.content),
            Some(&root),
            script.loc.start as u32,
            template.loc.start as u32,
            &crate::virtual_ts::VirtualTsOptions::default(),
            crate::virtual_ts::types::VirtualTsGenerationOptions {
                component_name: Some("Probe"),
                ..Default::default()
            },
        );
        let exports = output
            .mapping
            .rows()
            .filter(|row| row.meta.kind == ProjectionSpanKind::ComponentExport)
            .map(|row| {
                (
                    output.code.get(row.span.gen_range.clone()),
                    row.span.src_range.clone(),
                    row.meta.features.bits(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(exports, [(Some("Probe"), 0..0, 24)]);
    }

    #[test]
    fn component_export_only_serves_navigation_and_keeps_authored_completion() {
        let code = String::from("let value;\nexport default Name;\n");
        let start = code.find("Name").expect("exported component name");
        let output = publish(
            code,
            vec![VizeMapping::new(4..9, 12..17)],
            vec![],
            vec![],
            vec![],
            Some(start..start + 4),
        );
        assert_eq!(
            output
                .mapping
                .rows()
                .map(|row| row.meta)
                .collect::<Vec<_>>(),
            [
                ProjectionMeta::default(),
                ProjectionMeta {
                    features: ProjectionFeatures::DEFINITION.union(ProjectionFeatures::REFERENCES),
                    kind: ProjectionSpanKind::ComponentExport,
                },
            ]
        );
        for (feature, _) in ProjectionFeatures::NAMED {
            let expected = if feature == ProjectionFeatures::DEFINITION
                || feature == ProjectionFeatures::REFERENCES
            {
                Some(0)
            } else {
                None
            };
            assert_eq!(output.mapping.to_authored_for(start, feature), expected);
        }
        assert_eq!(
            output
                .mapping
                .to_generated_for(12, ProjectionFeatures::COMPLETION),
            Some(4)
        );
    }
}
