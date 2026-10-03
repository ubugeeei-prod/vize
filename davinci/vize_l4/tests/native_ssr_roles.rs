use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    id::NodeId,
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::{
    lang::js::NativeTemplateOwner,
    op::{Namespace, Op},
};
use vize_l3::decision::ssr::{SsrUnsupported, build_ssr_file_decisions};
use vize_l4::{
    targets::ssr::{SsrErrorKind, emit_file},
    write::Recorded,
};

#[test]
fn original_selected_search_elements_keep_neutral_structure_but_refuse_vue_ssr_role() {
    let mut captured = Vec::new();
    for (id, source, template, node) in [
        (
            "search-root",
            "<template><search/></template>",
            "<search/>",
            0,
        ),
        (
            "search-nested",
            "<template><div><search/></div></template>",
            "<div><search/></div>",
            1,
        ),
    ] {
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let component = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap())
            .unwrap()
            .unwrap();
        let mut owner =
            NativeTemplateOwner::new(component).unwrap_or_else(|_| panic!("original owner"));
        {
            let mut walk = owner.begin().unwrap();
            let selected = walk.selected();
            for child in selected.children() {
                walk.child(child).unwrap();
            }
            walk.complete().unwrap();
        }
        let output = owner.finish();
        let view = output.view().unwrap();
        let file = view.file().unwrap();
        let original = file.artifact().provenance().to_vec();
        let root = &file.artifact().root().ops[0];
        let Op::Element(root) = root else {
            panic!("neutral original HTML root")
        };
        let search = if root.tag == "search" {
            root
        } else {
            let Op::Element(child) = &root.children.ops[0] else {
                panic!("neutral original HTML child")
            };
            child
        };
        assert_eq!(search.tag, "search");
        assert_eq!(search.namespace, Namespace::Html);
        let analysis = build_ssr_file_decisions(file).unwrap();
        assert!(core::ptr::eq(analysis.file(), file));
        let error = emit_file::<Recorded>(&analysis).unwrap_err();
        assert_eq!(
            error.kind,
            SsrErrorKind::Unsupported(SsrUnsupported::ElementSemantics)
        );
        assert_eq!(error.node, NodeId::from_index(node));
        assert_eq!(
            source.get(error.span.start as usize..error.span.end as usize),
            Some("<search/>")
        );
        assert_eq!(analysis.artifact().provenance(), original);
        captured.push(serde_json::json!({"id":id,"source":source,"template":template,"outcome":"component_role_refusal","reason":"ElementSemantics","node":error.node.map(NodeId::index),"span":{"start":error.span.start,"end":error.span.end}}));
    }
    if let Ok(path) = std::env::var("VIZE_L4_SSR_NATIVE_CAPTURE") {
        let path = vize_l0::cstr!("{path}.refusals.json");
        std::fs::write(path.as_str(), serde_json::to_vec_pretty(&captured).unwrap()).unwrap();
    }
}
