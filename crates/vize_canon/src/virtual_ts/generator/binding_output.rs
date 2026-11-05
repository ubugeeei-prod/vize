//! Publish generated code with both binding metadata carriers.

use super::super::{VirtualTsOutput, VizeMapping, VizeSemanticLink};
use vize_carton::String;

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
