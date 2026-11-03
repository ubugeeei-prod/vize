//! Authored whole class module controls; runtime authority remains separately hosted.
use serde_json::{Value, json};
use vize_l0::{Allocator, Span};
use vize_l2::lang::js::{NativeTemplateIssue, NativeTemplateIssueKind};
use vize_l3::decision::vapor::{VaporUnsupported, build_native_vapor_file_decisions};
use vize_l4::{
    targets::vapor::{VaporErrorKind, emit_template},
    write::{NoLinks, Recorded},
};

#[test]
fn static_class_whitespace_entities_and_original_root_flags_emit_whole_modules() {
    let pack: Value = serde_json::from_str(include_str!(
        "../fixtures/native-vapor-static-class-controls.json"
    ))
    .unwrap();
    let mut captured = Vec::new();
    for row in pack["positive"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = row["source"].as_str().unwrap();
        let owner = crate::original(&arena, source);
        let analysis = build_native_vapor_file_decisions(owner.view().unwrap()).unwrap();
        assert!(core::ptr::eq(analysis.owner(), &owner));
        assert!(core::ptr::eq(analysis.artifact().source(), source));
        assert_eq!(analysis.vapor().unwrap().unsupported(), [], "{}", row["id"]);
        let recorded = emit_template::<Recorded>(&analysis).unwrap();
        let plain = emit_template::<NoLinks>(&analysis).unwrap();
        assert_eq!(
            recorded.text.as_str(),
            row["expectedCode"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        assert_eq!(recorded.text, plain.text);
        assert_eq!(recorded.helpers, plain.helpers);
        assert!(plain.links == NoLinks);
        let document = recorded.into_document();
        let map_text = document.source_map("NativeVaporClass.vue", source);
        let map: Value = serde_json::from_str(&map_text).unwrap();
        assert_eq!(map["sourcesContent"], json!([source]));
        assert!(!document.links().is_empty());
        let links: Vec<_> = document
            .links()
            .iter()
            .map(|link| {
                assert_eq!(link.generated.start, link.generated.end);
                assert_eq!(link.authored.start, link.authored.end);
                assert!(source.is_char_boundary(link.authored.start as usize));
                assert!(
                    document
                        .as_str()
                        .is_char_boundary(link.generated.start as usize)
                );
                json!({"generated":span(link.generated),"authored":span(link.authored)})
            })
            .collect();
        let component =
            vize_l4::targets::vapor::emit_component::<Recorded>(&analysis, "3.6.0-rc.9");
        let component = if analysis.vapor().unwrap().roots().len() > 1 {
            let error = component.unwrap_err();
            assert_eq!(error.kind, VaporErrorKind::ComponentFragmentLifecycle);
            json!({"code":null,"mapText":null,"map":null,"links":[],"error":format!("{error:#?}")})
        } else {
            let component = component.unwrap().into_document();
            let raw = component.source_map("NativeVaporClass.vue", source);
            json!({"code":component.as_str(),"mapText":raw,"map":serde_json::from_str::<Value>(&raw).unwrap(),
                "links":component.links().iter().map(|link|json!({"generated":span(link.generated),"authored":span(link.authored)})).collect::<Vec<_>>(),"error":null})
        };
        captured.push(
            json!({"id":row["id"],"source":source,"code":document.as_str(),
            "mapText":map_text,"map":map,"links":links,"component":component}),
        );
    }
    assert_eq!(captured.len(), 16);
    if let Ok(path) = std::env::var("VIZE_NATIVE_VAPOR_CAPTURE") {
        std::fs::write(
            format!("{path}.class-cases.json"),
            serde_json::to_vec_pretty(&captured).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn static_class_never_authorizes_nul_ordinary_cr_style_duplicates_or_dynamic_headers() {
    let pack: Value = serde_json::from_str(include_str!(
        "../fixtures/native-vapor-static-class-controls.json"
    ))
    .unwrap();
    for row in pack["negative"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = row["source"].as_str().unwrap();
        let descriptor = vize_l1::container::Vue.observe_descriptor(
            &arena,
            source,
            vize_l1::container::vue::DescriptorOptions {
                version: vize_l0::config::VueVersion::V3,
                dialect: vize_l0::config::VueDialect::Vue,
                template: vize_l1::markup::SurfaceParseOptions::default(),
            },
        );
        let selected = vize_l1::markup::NativeTemplateComponent::parse_in(
            &arena,
            descriptor.admitted().unwrap(),
        )
        .unwrap()
        .unwrap();
        let mut owner = vize_l2::lang::js::NativeTemplateOwner::new(selected)
            .map_err(|_| "owner")
            .unwrap();
        let mut refusal = None;
        {
            let mut walk = owner.begin().unwrap();
            let selected = walk.selected();
            for child in selected.children() {
                if let Err(issue) = walk.child(child) {
                    refusal = Some(issue);
                    break;
                }
            }
            if refusal.is_none() {
                walk.complete().unwrap();
            }
        }
        let owner = owner.finish();
        if row["requiredRefusal"] == "UnsupportedChild" {
            // Independently authored single-template input coordinates, not an
            // error-derived span or a successful L3 admission for refused L2.
            assert_eq!(&source[..10], "<template>");
            assert_eq!(&source[source.len() - 11..], "</template>");
            let span = Span::new(10, u32::try_from(source.len()).unwrap() - 11);
            assert_eq!(owner.selected().component().block().span(), span);
            assert_eq!(
                refusal,
                Some(NativeTemplateIssue {
                    span,
                    kind: NativeTemplateIssueKind::UnsupportedChild,
                }),
                "{}",
                row["id"]
            );
            assert!(owner.view().is_err());
            continue;
        }
        assert_eq!(refusal, None, "{}", row["id"]);
        let analysis = build_native_vapor_file_decisions(owner.view().unwrap()).unwrap();
        let expected = match row["requiredRefusal"].as_str().unwrap() {
            "DuplicateAttribute" => VaporUnsupported::DuplicateAttribute,
            "AttributeSemantics" => VaporUnsupported::AttributeSemantics,
            unexpected => panic!("unauthored negative boundary: {unexpected}"),
        };
        assert_eq!(
            emit_template::<Recorded>(&analysis).unwrap_err().kind,
            VaporErrorKind::Unsupported(expected)
        );
        assert_eq!(
            emit_template::<NoLinks>(&analysis).unwrap_err(),
            emit_template::<Recorded>(&analysis).unwrap_err()
        );
    }
}

fn span(value: Span) -> Value {
    json!({"start":value.start,"end":value.end})
}
