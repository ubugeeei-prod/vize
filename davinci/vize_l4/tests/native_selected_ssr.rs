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
use vize_l3::decision::ssr::{
    NativeSsrBuildError, SsrUnsupported, build_native_ssr_file_decisions,
};
use vize_l4::{
    module::{ModuleParts, RenderPlacement, RenderProperty, assemble, assemble_template},
    runtime::{Runtime, vocabulary},
    targets::ssr::{SsrErrorKind, emit_template},
    write::{NoLinks, Recorded},
};

#[expect(
    clippy::unwrap_used,
    reason = "invalid original fixture must fail admission"
)]
fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let component = {
        let descriptor = Vue.observe_descriptor(
            arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
            .unwrap()
            .unwrap()
    };
    let mut owner = NativeTemplateOwner::new(component).unwrap_or_else(|_| panic!("owner"));
    {
        let mut walk = owner.begin().unwrap();
        let selected = walk.selected();
        for child in selected.children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    core::hint::black_box(owner.finish())
}

#[test]
fn original_receipts_emit_eight_complete_pinned_modules_and_whole_source_maps() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-selected-ssr-vue-3.5.35.json")).unwrap();
    let mut modules = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let owner = completed(&arena, source);
        let file = owner.file().unwrap();
        let original = file.artifact().provenance().to_vec();
        let receipt = build_native_ssr_file_decisions(owner.view().unwrap()).unwrap();
        assert!(core::ptr::eq(receipt.owner(), &owner));
        assert!(core::ptr::eq(receipt.file(), file));
        assert!(core::ptr::eq(receipt.artifact().source(), source));
        assert_eq!(
            receipt.tables().nodes.len(),
            file.artifact().node_count() as usize
        );
        let recorded = assemble_template(
            emit_template::<Recorded>(&receipt).unwrap(),
            vocabulary(Runtime::VueServerRenderer),
        )
        .unwrap();
        let plain = assemble_template(
            emit_template::<NoLinks>(&receipt).unwrap(),
            vocabulary(Runtime::VueServerRenderer),
        )
        .unwrap();
        assert_eq!(recorded.text, fixture["code"].as_str().unwrap());
        assert_eq!(plain.text, recorded.text);
        assert_eq!(plain.helpers, recorded.helpers);
        let document = recorded.into_document();
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
            serde_json::from_str(&document.source_map("NativeSelectedSsr.vue", source)).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        let mut parts =
            ModuleParts::for_runtime(Runtime::VueServerRenderer, "3.5.35", "__sfc__").unwrap();
        parts.render = Some(emit_template::<Recorded>(&receipt).unwrap());
        parts.placement = RenderPlacement::Function {
            binding: "ssrRender",
            property: RenderProperty::SsrRender,
        };
        let sfc = assemble(parts).unwrap().into_document();
        let sfc_map: serde_json::Value =
            serde_json::from_str(&sfc.source_map("NativeSelectedSsr.vue", source)).unwrap();
        assert_eq!(sfc_map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(receipt.artifact().provenance(), original);
        modules.push(serde_json::json!({"id":fixture["id"],"source":source,"template":fixture["template"],"outcome":"complete_original_module","code":document.as_str(),"map":map,"sfcCode":sfc.as_str(),"sfcMap":sfc_map}));
    }
    let mut refusals = Vec::new();
    for (id, source, node) in [
        ("search-root", "<template><search/></template>", 0),
        (
            "search-nested",
            "<template><div><search/></div></template>",
            1,
        ),
    ] {
        let arena = Allocator::default();
        let owner = completed(&arena, source);
        let receipt = build_native_ssr_file_decisions(owner.view().unwrap()).unwrap();
        let error = emit_template::<Recorded>(&receipt).unwrap_err();
        assert_eq!(
            error.kind,
            SsrErrorKind::Unsupported(SsrUnsupported::ElementSemantics)
        );
        assert_eq!(error.node.unwrap().index(), node);
        assert_eq!(
            source.get(error.span.start as usize..error.span.end as usize),
            Some("<search/>")
        );
        refusals.push(serde_json::json!({"id":id,"source":source,"outcome":"original_role_refusal","reason":"ElementSemantics","node":error.node.map(|node| node.index()),"span":{"start":error.span.start,"end":error.span.end}}));
    }
    for (id, source) in [
        ("global-style", "<template><div/></template><style></style>"),
        (
            "scoped-style",
            "<style scoped>div{color:red}</style><template><div/></template>",
        ),
    ] {
        let arena = Allocator::default();
        let owner = completed(&arena, source);
        let Err(NativeSsrBuildError::StyledSource { span }) =
            build_native_ssr_file_decisions(owner.view().unwrap())
        else {
            panic!("styled source cannot mint output authority");
        };
        assert_eq!(span.start, 0);
        assert_eq!(span.end as usize, source.len());
        refusals.push(serde_json::json!({"id":id,"source":source,"outcome":"original_style_refusal","reason":"StyledSource","span":{"start":span.start,"end":span.end}}));
    }
    if let Ok(path) = std::env::var("VIZE_L4_SELECTED_SSR_NATIVE_CAPTURE") {
        let capture = serde_json::json!({"custody":"original_completed_template","modules":modules,"refusals":refusals});
        std::fs::write(path, serde_json::to_vec_pretty(&capture).unwrap()).unwrap();
    }
}
