use super::super::protocol::protocol_semantic_links;

#[test]
fn protocol_v1_omits_internal_navigation_links() {
    use crate::virtual_ts::{VizeSemanticLink, VizeSemanticLinkKind};

    let links: Vec<_> = [
        VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
        VizeSemanticLinkKind::VueComponentPropNavigation,
        VizeSemanticLinkKind::VuePlainScriptExport,
        VizeSemanticLinkKind::VueOptionsApiBinding,
    ]
    .into_iter()
    .map(|kind| VizeSemanticLink {
        source_range: 1..6,
        target_range: 10..15,
        kind,
    })
    .collect();

    let protocol = protocol_semantic_links(&links);

    assert_eq!(
        serde_json::to_value(protocol).unwrap(),
        serde_json::json!([{
            "sourceStart": 1, "sourceLength": 5, "targetStart": 10, "targetLength": 5,
            "kind": "vueSetupTemplateRefUnwrap"
        }])
    );
}
