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
            super::super::mapping::ProjectionMeta::of_kind(
                super::super::mapping::ProjectionSpanKind::ComponentExport,
            ),
        );
    }
    mapping.set_prop_default_key_links(prop_default_key_links);
    super::super::mapping::publish_virtual_ts(VirtualTsOutput { code: ts, mapping })
}
