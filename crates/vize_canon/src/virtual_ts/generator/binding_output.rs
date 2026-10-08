//! Publish generated code with both binding metadata carriers.

use super::super::{VirtualTsOutput, VizeMapping, VizeSemanticLink};
use vize_carton::String;

pub(super) fn publish(
    ts: String,
    mappings: Vec<VizeMapping>,
    semantic_links: Vec<VizeSemanticLink>,
    prop_default_key_links: Vec<VizeSemanticLink>,
) -> VirtualTsOutput {
    let mut mapping =
        super::super::mapping::ProjectionMapping::from_parts(mappings, semantic_links);
    mapping.set_prop_default_key_links(prop_default_key_links);
    super::super::mapping::publish_virtual_ts(VirtualTsOutput { code: ts, mapping })
}
