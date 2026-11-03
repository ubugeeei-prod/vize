//! Retained value shape changes formatting without inventing semantic facts.

use super::{bindings, fixture, resolutions};
use crate::{
    expr::{
        ResolvedExpressions,
        vue::{AccessStyle, VueAccess},
    },
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::emit,
    write::{NoLinks, Recorded},
};
use alloc::vec::Vec;
use vize_l0::Allocator;
use vize_l3::decision::{build_dom_decisions, dom::ContextOnly};

#[test]
fn complete_bare_property_modules_match_pinned_vue_and_keep_named_links() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/dom-props-vue-3.5.35.json"
    ))
    .unwrap();
    let mut captured = Vec::new();
    for reference in pack["fixtures"].as_array().unwrap() {
        if reference["family"] != "bare" {
            continue;
        }
        let arena = Allocator::default();
        let source = reference["source"].as_str().unwrap();
        let artifact =
            fixture::property_value(&arena, source, reference["expression"].as_str().unwrap());
        let tables = resolutions(&artifact);
        let bindings = bindings();
        let ids: Vec<_> = bindings.iter().map(|binding| binding.id).collect();
        let analysis = build_dom_decisions(&artifact, &ContextOnly::new(&tables, &ids)).unwrap();
        let access = VueAccess::checked(
            &bindings,
            AccessStyle::Function,
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        let expressions = ResolvedExpressions::checked(&tables).unwrap();
        let recorded = assemble_template(
            emit::<Recorded>(&analysis, expressions, &access).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        let plain = assemble_template(
            emit::<NoLinks>(&analysis, expressions, &access).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        assert_eq!(
            recorded.text.as_str(),
            reference["code"].as_str().unwrap(),
            "{}",
            reference["id"]
        );
        assert_eq!(plain.text, recorded.text);
        assert_eq!(plain.helpers, recorded.helpers);
        let document = recorded.into_document();
        let link = document
            .links()
            .iter()
            .find(|link| {
                link.name
                    .as_ref()
                    .is_some_and(|name| name.as_str() == "msg")
            })
            .unwrap();
        assert_eq!(
            source.get(link.authored.start as usize..link.authored.end as usize),
            Some("msg")
        );
        assert_eq!(
            document
                .as_str()
                .get(link.generated.start as usize..link.generated.end as usize),
            Some("_ctx.msg")
        );
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("PropsDom.vue", source)).unwrap();
        assert_eq!(map["names"], serde_json::json!(["msg"]));
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        captured.push(serde_json::json!({"id": reference["id"], "source": source, "code": document.as_str(), "map": map}));
    }
    assert_eq!(captured.len(), 7);
    if let Ok(path) = std::env::var("VIZE_L4_PROPS_BARE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captured).unwrap()).unwrap();
    }
}
