//! Whole original-selected modules, independent maps and all-or-nothing refusal.
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::{NativeTemplateFile, NativeTemplateOwner};
use vize_l3::decision::vapor::{VaporUnsupported, build_native_vapor_file_decisions};
use vize_l4::{
    targets::vapor::{VaporErrorKind, emit_template},
    write::{NoLinks, Recorded},
};

fn original<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut owner = NativeTemplateOwner::new(selected)
        .map_err(|_| "original owner")
        .unwrap();
    {
        let mut walk = owner.begin().unwrap();
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    owner.finish()
}

#[test]
fn twelve_original_modules_and_complete_maps_match_pinned_fixtures() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-vapor-vue-3.6.0-rc.9.json")).unwrap();
    assert_eq!(pack["version"], "3.6.0-rc.9");
    let mut captured = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let owner = original(&arena, source);
        let analysis = build_native_vapor_file_decisions(owner.view().unwrap()).unwrap();
        assert!(core::ptr::eq(analysis.owner(), &owner));
        assert!(core::ptr::eq(analysis.artifact().source(), source));
        assert_eq!(analysis.vapor().unwrap().unsupported(), []);
        let recorded = emit_template::<Recorded>(&analysis).unwrap();
        let plain = emit_template::<NoLinks>(&analysis).unwrap();
        assert_eq!(
            recorded.text.as_str(),
            fixture["code"].as_str().unwrap(),
            "{}",
            fixture["id"]
        );
        assert_eq!(recorded.text, plain.text);
        assert_eq!(recorded.helpers, plain.helpers);
        assert!(plain.links == NoLinks);
        let document = recorded.into_document();
        let anchors = fixture["anchors"].as_array().unwrap();
        assert_eq!(document.links().len(), anchors.len());
        for (link, anchor) in document.links().iter().zip(anchors) {
            assert_eq!(link.generated.start, link.generated.end);
            assert_eq!(link.authored.start, link.authored.end);
            assert_eq!(
                u64::from(link.generated.start),
                anchor["generated"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(link.authored.start),
                anchor["source"].as_u64().unwrap()
            );
            assert!(source.is_char_boundary(link.authored.start as usize));
            assert!(
                document
                    .as_str()
                    .is_char_boundary(link.generated.start as usize)
            );
        }
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("NativeVapor.vue", source)).unwrap();
        assert_eq!(map, fixture["map"], "{}", fixture["id"]);
        captured.push(serde_json::json!({ "id":fixture["id"], "source":source,
            "code":document.as_str(), "map":map, "nodes":analysis.artifact().node_count(),
            "roots":analysis.vapor().unwrap().roots().len() }));
    }
    assert_eq!(captured.len(), 12);
    if let Ok(path) = std::env::var("VIZE_NATIVE_VAPOR_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captured).unwrap()).unwrap();
    }
}

#[test]
fn original_target_refusals_never_return_a_partial_module() {
    for (template, reason) in [
        (
            "<div>before</div><p>after</p>",
            VaporUnsupported::ElementSemantics,
        ),
        (
            "<div>before</div><span hidden>after</span>",
            VaporUnsupported::AttributeSemantics,
        ),
        (
            "<div>before</div><span title=\"a\r b\">after</span>",
            VaporUnsupported::AttributeSemantics,
        ),
    ] {
        let arena = Allocator::default();
        let source = format!("<template>{template}</template>");
        let owner = original(&arena, &source);
        let analysis = build_native_vapor_file_decisions(owner.view().unwrap()).unwrap();
        let recorded = emit_template::<Recorded>(&analysis).unwrap_err();
        let plain = emit_template::<NoLinks>(&analysis).unwrap_err();
        assert_eq!(recorded, plain);
        assert_eq!(recorded.kind, VaporErrorKind::Unsupported(reason));
        assert!(recorded.node.is_some());
        assert!(
            source
                .get(recorded.span.start as usize..recorded.span.end as usize)
                .is_some()
        );
    }
}

#[test]
fn unsupported_original_lower_events_cannot_authorize_the_target() {
    for template in [
        "<span>a b</span>",
        "<span>a&amp;b</span>",
        "<span :title=\"1\">x</span>",
        "<span class=\"x\">x</span>",
        "<span>{{1}}</span>",
    ] {
        let arena = Allocator::default();
        let source = format!("<template>{template}</template>");
        let descriptor = Vue.observe_descriptor(
            &arena,
            &source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let selected = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
            .unwrap()
            .unwrap();
        let mut owner = NativeTemplateOwner::new(selected)
            .map_err(|_| "original owner")
            .unwrap();
        {
            let mut walk = owner.begin().unwrap();
            let selected = walk.selected();
            assert!(walk.child(selected.children().next().unwrap()).is_err());
        }
        let refused = owner.finish();
        assert!(refused.view().is_err(), "{template}");
    }
}
