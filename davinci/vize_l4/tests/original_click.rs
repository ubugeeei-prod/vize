//! Real original-template/File/On owners produce complete pinned click modules.

#[path = "original_click/refusal.rs"]
mod refusal;

use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::{
    lang::js::{NativeTemplateFile, NativeTemplateOwner},
    op::{BindingOp, OnOp, Op, Region},
};
use vize_l3::decision::{dom::PropertyRole, native::build_native_dom_file_decisions};
use vize_l4::{
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::emit_template,
    write::{NoLinks, Recorded},
};

fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
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
    let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("owner"));
    {
        let mut walk = owner.begin().unwrap();
        for child in walk.selected().children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    owner.finish()
}

fn on<'f, 'a>(region: &'f Region<'a>) -> &'f OnOp<'a> {
    for op in &region.ops {
        if let Op::Element(element) = op {
            for binding in &element.bindings {
                if let BindingOp::On(on) = binding {
                    return on;
                }
            }
            if !element.children.ops.is_empty() {
                return on(&element.children);
            }
        }
    }
    panic!("actual original On")
}

#[test]
fn five_original_click_modules_match_the_complete_pinned_compiler_bytes() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/original-click-vue-3.5.35.json")).unwrap();
    assert_eq!(pack["fixtures"].as_array().unwrap().len(), 5);
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let owner = completed(&arena, source);
        let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
        let file = analysis.file();
        assert!(core::ptr::eq(analysis.owner(), &owner));
        assert!(core::ptr::eq(file.artifact().source(), source));
        let original_on = on(file.artifact().root());
        let lower = file.handler_for(original_on).unwrap();
        let row = analysis.handler(lower.id().node()).unwrap();
        assert!(row.accepts_on(original_on));
        assert!(core::ptr::eq(row.handler().file(), file));
        assert!(core::ptr::eq(row.resolution(), lower.resolution().unwrap()));
        assert!(row.resolution().syntax().leading_declaration());
        assert_eq!(row.resolution().syntax().first_return(), None);
        assert!(!row.resolution().references().is_empty());
        let facts = analysis.dom().unwrap();
        assert_eq!(facts.unsupported(), []);
        assert_eq!(
            facts.binding(lower.id().node()).unwrap().role,
            PropertyRole::Event
        );
        let recorded = assemble_template(
            emit_template::<Recorded>(&analysis).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        let plain = assemble_template(
            emit_template::<NoLinks>(&analysis).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        assert_eq!(
            recorded.text.as_str(),
            fixture["code"].as_str().unwrap(),
            "{}",
            fixture["id"]
        );
        assert_eq!(plain.text, recorded.text);
        assert_eq!(plain.helpers, recorded.helpers);
        let document = recorded.into_document();
        let syntax = row.resolution().input().operand().syntax();
        assert_eq!(syntax.diagnostics().count(), 0);
        assert!(core::ptr::eq(syntax.source().authored_root(), source));
        let body_span = syntax.source().span();
        let body_links: Vec<_> = document
            .links()
            .iter()
            .filter(|link| {
                link.authored.start >= body_span.start && link.authored.end <= body_span.end
            })
            .collect();
        assert_eq!(body_links.first().unwrap().authored.start, body_span.start);
        assert_eq!(body_links.last().unwrap().authored.end, body_span.end);
        assert!(
            body_links
                .windows(2)
                .all(|pair| pair[0].authored.end == pair[1].authored.start)
        );
        let decoded: String = body_links
            .iter()
            .map(|link| link.generated.slice(document.as_str()))
            .collect();
        assert_eq!(decoded, syntax.source().text());
        if let Some(map) = syntax.source().decode_map() {
            for segment in map
                .segments()
                .iter()
                .filter(|segment| segment.authored().slice(source).starts_with('&'))
            {
                assert!(body_links.iter().any(|link| {
                    link.authored == segment.authored()
                        && link.name.is_none()
                        && link.generated.slice(document.as_str())
                            == segment.decoded().slice(syntax.source().text())
                }));
            }
        }
        for reference in row.resolution().references() {
            let authored = row.resolution().authored_span(reference.span).unwrap();
            assert!(body_links.iter().any(|link| link.authored == authored
                && link.name.as_deref() == Some("$event")
                && link.generated.slice(document.as_str()) == "$event"));
        }
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
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("Click.vue", source)).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(map["names"], serde_json::json!(["$event"]));
    }
}
