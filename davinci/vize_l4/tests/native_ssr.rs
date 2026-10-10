use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::Lang;
use vize_l1_to_l2::native::NativeComponent;
use vize_l2::file::{Declaration, TemplatePolicy, TemplateScope};
use vize_l2::lang::js::FileProducer;
use vize_l3::decision::ssr::{SsrUnsupported, build_ssr_file_decisions};
use vize_l3::decision::{build_decisions, policy::TargetPolicy};
use vize_l4::module::{ModuleParts, RenderPlacement, RenderProperty, assemble, assemble_template};
use vize_l4::runtime::{Runtime, vocabulary};
use vize_l4::targets::ssr::{SsrErrorKind, emit, emit_file};
use vize_l4::write::{NoLinks, Recorded};

#[derive(Clone, Copy)]
struct DiagnosticValues;
impl TemplatePolicy for DiagnosticValues {
    fn visible(self, _: &Declaration) -> bool {
        true
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "invalid original dev fixture must fail construction"
)]
fn native_file<'a>(arena: &'a Allocator, source: &'a str) -> vize_l2::file::FileArtifact<'a> {
    let block = SourceRoot::new(source).unwrap().block(source, 0).unwrap();
    let component = NativeComponent::parse_in(arena, block).unwrap();
    let mut producer = FileProducer::new(arena, source).unwrap();
    {
        // This genuine lower File guard records normal end; it confers no
        // original SFC custody, script role or runtime expression exposure.
        let mut region = producer
            .template_region(TemplateScope::Root, DiagnosticValues)
            .unwrap();
        let mut walk = region.walk(block.span()).unwrap();
        let produced = component.construct_in(&mut walk, Lang::Js).unwrap();
        assert!(produced.is_supported(), "{source}: {:?}", produced.holes);
        walk.complete().unwrap();
    }
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    file
}

#[test]
fn native_complete_modules_match_pinned_ssr_function_bytes_and_original_maps() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-ssr-vue-3.5.35.json")).unwrap();
    let mut captured = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let id = fixture["id"].as_str().unwrap();
        let file = native_file(&arena, source);
        let original = file.artifact().provenance().to_vec();
        let analysis = build_ssr_file_decisions(&file).unwrap();
        assert!(core::ptr::eq(analysis.file(), &file));
        assert_eq!(
            analysis.tables().nodes.len(),
            file.artifact().node_count() as usize
        );
        let recorded = assemble_template(
            emit_file::<Recorded>(&analysis).unwrap(),
            vocabulary(Runtime::VueServerRenderer),
        )
        .unwrap();
        let plain = assemble_template(
            emit_file::<NoLinks>(&analysis).unwrap(),
            vocabulary(Runtime::VueServerRenderer),
        )
        .unwrap();
        assert_eq!(recorded.text, fixture["code"].as_str().unwrap(), "{id}");
        assert_eq!(plain.text, recorded.text, "{id}: both sinks");
        assert_eq!(plain.helpers, recorded.helpers);
        let document = recorded.into_document();
        for link in document.links() {
            assert!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some(),
                "{id}: authored"
            );
            assert!(
                document
                    .as_str()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    .is_some(),
                "{id}: generated"
            );
        }
        if !source.is_empty() {
            assert!(
                !document.links().is_empty(),
                "{id}: original structural links"
            );
        }
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("NativeSsr.vue", source)).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(map["sources"], serde_json::json!(["NativeSsr.vue"]));
        assert_eq!(file.artifact().provenance(), original);
        let mut parts =
            ModuleParts::for_runtime(Runtime::VueServerRenderer, "3.5.35", "__sfc__").unwrap();
        parts.render = Some(emit_file::<Recorded>(&analysis).unwrap());
        parts.placement = RenderPlacement::Function {
            binding: "ssrRender",
            property: RenderProperty::SsrRender,
        };
        let sfc = assemble(parts).unwrap().into_document();
        let sfc_map: serde_json::Value =
            serde_json::from_str(&sfc.source_map("NativeSsr.vue", source)).unwrap();
        let links = |document: &vize_l4::write::EmitDocument| {
            document.links().iter().map(|link| {
                serde_json::json!({"authored":{"start":link.authored.start,"end":link.authored.end},"generated":{"start":link.generated.start,"end":link.generated.end},"name":link.name.as_deref(),"segment":link.segment})
            }).collect::<Vec<_>>()
        };
        captured.push(serde_json::json!({"id": id, "source": source, "filename":"NativeSsr.vue", "code": document.as_str(), "map": map, "links":links(&document), "sfcCode": sfc.as_str(), "sfcMap": sfc_map, "sfcLinks":links(&sfc), "outcome": "complete_module"}));
    }
    assert_eq!(captured.len(), 25);
    if let Ok(path) = std::env::var("VIZE_L4_SSR_NATIVE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captured).unwrap()).unwrap();
    }
}

#[test]
fn wrong_policy_and_unsupported_whole_file_return_no_writer() {
    let arena = Allocator::default();
    let file = native_file(&arena, "<div/>");
    let dom = build_decisions(file.artifact(), TargetPolicy::Dom).unwrap();
    assert_eq!(
        emit::<Recorded>(&dom).unwrap_err().kind,
        SsrErrorKind::WrongPolicy
    );
    let source = "<div :id=\"1\"/>";
    let file = native_file(&arena, source);
    let original = file.artifact().provenance().to_vec();
    let analysis = build_ssr_file_decisions(&file).unwrap();
    let error = emit_file::<Recorded>(&analysis).unwrap_err();
    assert_eq!(
        error.kind,
        SsrErrorKind::Unsupported(SsrUnsupported::Binding)
    );
    assert_eq!(
        source.get(error.span.start as usize..error.span.end as usize),
        Some(":id=\"1\"")
    );
    assert!(error.node.is_some());
    assert_eq!(analysis.artifact().provenance(), original);
    for source in [
        "<div style=\"color:red\"/>",
        "<input value=\"a\">",
        "<input true-value=\"a\">",
        "<input false-value=\"a\">",
    ] {
        let file = native_file(&arena, source);
        let analysis = build_ssr_file_decisions(&file).unwrap();
        assert_eq!(
            emit_file::<NoLinks>(&analysis).unwrap_err().kind,
            SsrErrorKind::Unsupported(SsrUnsupported::AttributeSemantics)
        );
    }
}

#[test]
fn sfc_module_assembly_preserves_ssr_attachment_and_whole_source_links() {
    let arena = Allocator::default();
    let source = "<div><p title=\"&quot;&amp;\">猫&amp;</p></div>";
    let file = native_file(&arena, source);
    let analysis = build_ssr_file_decisions(&file).unwrap();
    let mut parts =
        ModuleParts::for_runtime(Runtime::VueServerRenderer, "3.5.35", "__sfc__").unwrap();
    parts.render = Some(emit_file::<Recorded>(&analysis).unwrap());
    parts.placement = RenderPlacement::Function {
        binding: "ssrRender",
        property: RenderProperty::SsrRender,
    };
    let document = assemble(parts).unwrap().into_document();
    assert_eq!(
        document.as_str(),
        concat!(
            "import { ssrRenderAttrs as _ssrRenderAttrs } from \"@vue/server-renderer\"\n",
            "const __sfc__ = {}\n;\n",
            "function ssrRender(_ctx, _push, _parent, _attrs) {\n",
            "  _push(`<div${_ssrRenderAttrs(_attrs)}><p title=\"&quot;&amp;\">猫&amp;</p></div>`)\n",
            "}\n__sfc__.ssrRender = ssrRender\nexport default __sfc__\n",
        )
    );
    let link = document
        .links()
        .iter()
        .find(|link| {
            source.get(link.authored.start as usize..link.authored.end as usize) == Some("猫&amp;")
        })
        .unwrap();
    assert_eq!(
        document
            .as_str()
            .get(link.generated.start as usize..link.generated.end as usize),
        Some("猫&amp;")
    );
    assert_ne!(link.generated.start, 0);
    assert!(link.generated.end > link.generated.start);
    let start = source.find("猫&amp;").unwrap() as u32;
    assert_eq!(
        link.authored,
        Span::new(start, start + "猫&amp;".len() as u32)
    );
}
