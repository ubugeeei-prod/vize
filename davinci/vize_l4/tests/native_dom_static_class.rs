//! Static class output from a genuinely completed lower File, before SFC admission.

use vize_l0::{Allocator, SourceRoot};
use vize_l1::embed::Lang;
use vize_l1_to_l2::native::NativeComponent;
use vize_l2::{
    file::{Declaration, TemplatePolicy, TemplateScope},
    lang::js::FileProducer,
};
use vize_l3::decision::{build_dom_file_decisions, dom::DomUnsupported};
use vize_l4::{
    module::{ModuleParts, RenderPlacement, RenderProperty, assemble, assemble_template},
    runtime::{Runtime, vocabulary},
    targets::dom::{DomErrorKind, emit_file},
    write::{EmitDocument, NoLinks, Recorded},
};

#[derive(Clone, Copy)]
struct DiagnosticValues;
impl TemplatePolicy for DiagnosticValues {
    fn visible(self, _: &Declaration) -> bool {
        true
    }
}

fn file<'a>(arena: &'a Allocator, source: &'a str) -> vize_l2::file::FileArtifact<'a> {
    let block = SourceRoot::new(source).unwrap().block(source, 0).unwrap();
    let native = NativeComponent::parse_in(arena, block).unwrap();
    let mut producer = FileProducer::new(arena, source).unwrap();
    {
        let mut region = producer
            .template_region(TemplateScope::Root, DiagnosticValues)
            .unwrap();
        let mut walk = region.walk(block.span()).unwrap();
        assert!(
            native
                .construct_in(&mut walk, Lang::Js)
                .unwrap()
                .is_supported()
        );
        walk.complete().unwrap();
    }
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    file
}

fn links(document: &EmitDocument) -> Vec<serde_json::Value> {
    document
        .links()
        .iter()
        .map(|link| {
            serde_json::json!({
                "authored":{"start":link.authored.start,"end":link.authored.end},
                "generated":{"start":link.generated.start,"end":link.generated.end},
                "name":link.name.as_deref(),"segment":link.segment
            })
        })
        .collect()
}

#[test]
fn complete_static_class_modules_match_official_bytes_and_original_links() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/native-dom-static-class-vue-3.5.35.json"
    ))
    .unwrap();
    let mut captures = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let file = file(&arena, source);
        let provenance = file.artifact().provenance().to_vec();
        let analysis = build_dom_file_decisions(&file).unwrap();
        assert!(
            analysis.dom().unwrap().unsupported().is_empty(),
            "{}",
            fixture["id"]
        );
        let recorded = assemble_template(
            emit_file::<Recorded>(&analysis).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        let plain = assemble_template(
            emit_file::<NoLinks>(&analysis).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        assert_eq!(
            recorded.text,
            fixture["code"].as_str().unwrap(),
            "{}",
            fixture["id"]
        );
        assert_eq!(recorded.text, plain.text);
        assert_eq!(recorded.helpers, plain.helpers);
        let document = recorded.into_document();
        let mut parts = ModuleParts::for_runtime(Runtime::VueDom, "3.5.35", "__sfc__").unwrap();
        parts.render = Some(emit_file::<Recorded>(&analysis).unwrap());
        parts.placement = RenderPlacement::Function {
            binding: "render",
            property: RenderProperty::Render,
        };
        let component = assemble(parts).unwrap().into_document();
        for document in [&document, &component] {
            assert!(!document.links().is_empty());
            for link in document.links() {
                assert!(
                    source
                        .get(link.authored.start as usize..link.authored.end as usize)
                        .is_some()
                );
                assert!(
                    document
                        .as_str()
                        .get(link.generated.start as usize..link.generated.end as usize)
                        .is_some()
                );
            }
        }
        assert_eq!(file.artifact().provenance(), provenance);
        captures.push(serde_json::json!({
            "id":fixture["id"],"source":source,"filename":"NativeDomClass.vue",
            "code":document.as_str(),"map":serde_json::from_str::<serde_json::Value>(&document.source_map("NativeDomClass.vue",source)).unwrap(),"links":links(&document),
            "sfcCode":component.as_str(),"sfcMap":serde_json::from_str::<serde_json::Value>(&component.source_map("NativeDomClass.vue",source)).unwrap(),"sfcLinks":links(&component),
            "outcome":"complete_module"
        }));
    }
    assert_eq!(captures.len(), 16);
    if let Ok(path) = std::env::var("VIZE_L4_DOM_STATIC_CLASS_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

#[test]
fn unimplemented_special_and_combined_class_semantics_never_return_a_writer() {
    for (source, reason) in [
        (
            "<div class=\"ready\" style=\"color:red\"/>",
            DomUnsupported::SpecialAttribute,
        ),
        (
            "<div class=\"ready\" :class=\"'active'\"/>",
            DomUnsupported::DuplicateProperty,
        ),
        (
            "<div :class=\"'active'\" class=\"ready\"/>",
            DomUnsupported::DuplicateProperty,
        ),
    ] {
        let arena = Allocator::default();
        let file = file(&arena, source);
        let analysis = build_dom_file_decisions(&file).unwrap();
        let recorded = emit_file::<Recorded>(&analysis).unwrap_err();
        assert_eq!(recorded, emit_file::<NoLinks>(&analysis).unwrap_err());
        assert_eq!(recorded.kind, DomErrorKind::Unsupported(reason));
    }
}
