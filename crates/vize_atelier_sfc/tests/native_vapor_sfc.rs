//! Genuine whole-SFC source custody, complete modules/maps and target refusals.
use vize_atelier_sfc::{
    NativeVaporSfcCompileError, NativeVaporSfcCompileOptions, compile_native_vapor_sfc,
};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1_to_l2::native_file::NativeSelectedSfcIssueKind;
use vize_l3::decision::vapor::build_native_vapor_file_decisions;
use vize_l4::{
    module::AssemblyError,
    targets::vapor::{VaporErrorKind, emit_component},
    write::{NoLinks, Recorded},
};

#[test]
fn whole_original_components_and_maps_match_complete_independent_fixtures() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native_vapor_sfc_vue_3_6_rc9.json")).unwrap();
    assert_eq!(pack["version"], "3.6.0-rc.9");
    let mut captures = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let options = NativeVaporSfcCompileOptions {
            filename: "NativeVapor.vue",
            source_map: true,
            ..Default::default()
        };
        let compilation = compile_native_vapor_sfc(&arena, source, options);
        let output = compilation.result().unwrap();
        let plain = compile_native_vapor_sfc(
            &arena,
            source,
            NativeVaporSfcCompileOptions {
                source_map: false,
                ..options
            },
        );
        assert_eq!(plain.result().unwrap().code(), output.code());
        assert!(plain.result().unwrap().source_map().is_none());
        assert!(plain.result().unwrap().document().links().is_empty());
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source
        ));
        let original = compilation.observation().template().unwrap();
        let file = original.file().unwrap();
        assert!(file.is_complete());
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert!(core::ptr::eq(
            original.selected().component().block().root_source(),
            source
        ));
        let analysis = build_native_vapor_file_decisions(
            compilation
                .observation()
                .admitted()
                .unwrap()
                .into_template_view(),
        )
        .unwrap();
        assert!(core::ptr::eq(analysis.owner(), original));
        let recorded = emit_component::<Recorded>(&analysis, "3.6.0-rc.9").unwrap();
        let no_links = emit_component::<NoLinks>(&analysis, "3.6.0-rc.9").unwrap();
        assert_eq!(recorded.text, no_links.text);
        assert_eq!(recorded.helpers, no_links.helpers);
        assert_eq!(recorded.text.as_str(), output.code());
        assert_eq!(
            output.code(),
            fixture["code"].as_str().unwrap(),
            "{}",
            fixture["id"]
        );
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        assert_eq!(map, fixture["map"], "{}", fixture["id"]);
        let links = output.document().links();
        let anchors = fixture["anchors"].as_array().unwrap();
        assert_eq!(links.len(), anchors.len());
        for (link, anchor) in links.iter().zip(anchors) {
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
                output
                    .code()
                    .is_char_boundary(link.generated.start as usize)
            );
        }
        captures.push(serde_json::json!({"id":fixture["id"], "source":source,
            "code":output.code(), "map":map, "roots":analysis.vapor().unwrap().roots().len(),
            "nodes":file.artifact().node_count()}));
    }
    assert_eq!(captures.len(), 11);
    if let Some(path) = std::env::var_os("VIZE_NATIVE_VAPOR_SFC_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

#[test]
fn unsupported_whole_source_envelopes_preserve_authentic_observations() {
    for (source, expected) in [
        (
            "<script></script><template><div/></template>",
            NativeSelectedSfcIssueKind::Script(vize_l1::container::vue::ScriptRole::Ordinary),
        ),
        (
            "<script setup lang=ts>const value:number=1</script><template><div/></template>",
            NativeSelectedSfcIssueKind::Script(vize_l1::container::vue::ScriptRole::Setup),
        ),
        (
            "<template><div/></template><style></style>",
            NativeSelectedSfcIssueKind::Style,
        ),
        (
            "<template><div/></template><style scoped>.x{color:red}</style>",
            NativeSelectedSfcIssueKind::Style,
        ),
        (
            "<template src='./missing.html'></template>",
            NativeSelectedSfcIssueKind::Descriptor,
        ),
        (
            "<template><div/></template><i18n>{}</i18n>",
            NativeSelectedSfcIssueKind::Descriptor,
        ),
        (
            "<template lang=pug>div</template>",
            NativeSelectedSfcIssueKind::Descriptor,
        ),
        (
            "<template vapor><div/></template>",
            NativeSelectedSfcIssueKind::Descriptor,
        ),
        ("<!--no template-->", NativeSelectedSfcIssueKind::Descriptor),
    ] {
        let arena = Allocator::default();
        let compilation = compile_native_vapor_sfc(&arena, source, Default::default());
        let Err(NativeVaporSfcCompileError::Source(issue)) = compilation.result() else {
            panic!("source refusal: {source}");
        };
        assert_eq!(issue.kind, expected, "{source}");
        assert_eq!(compilation.observation().issues()[0], issue);
        assert!(compilation.observation().admitted().is_none());
        assert!(compilation.observation().template().is_none());
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source
        ));
    }
}

#[test]
fn structural_completion_never_grants_unsupported_vapor_semantics() {
    for template in [
        r#"<div @click="var unused=$event;"/>"#,
        r#"<div v-for="item in 2"/>"#,
        "<template><div/></template>",
        "<slot/>",
        "<div><span>a b</span></div>",
        r#"<div><span :title="1"/></div>"#,
        "<div>before</div><search>after</search>",
    ] {
        let arena = Allocator::default();
        let source = format!("<template>{template}</template>");
        for source_map in [false, true] {
            let compilation = compile_native_vapor_sfc(
                &arena,
                &source,
                NativeVaporSfcCompileOptions {
                    source_map,
                    ..Default::default()
                },
            );
            assert!(compilation.result().is_err(), "{template}");
            assert!(core::ptr::eq(
                compilation.observation().descriptor().source(),
                source.as_str()
            ));
        }
    }
}

#[test]
fn unaudited_runtime_and_source_profiles_have_no_fallback() {
    let source = "<template><div/></template>";
    let arena = Allocator::default();
    let options = NativeVaporSfcCompileOptions {
        runtime_version: "3.5.35",
        ..Default::default()
    };
    let compilation = compile_native_vapor_sfc(&arena, source, options);
    assert!(
        matches!(compilation.result(), Err(NativeVaporSfcCompileError::Vapor(error)) if error.kind == VaporErrorKind::Assembly(AssemblyError::UnsupportedRuntimeVersion))
    );
    for (version, dialect) in [
        (VueVersion::V2, VueDialect::Vue),
        (VueVersion::V3, VueDialect::PetiteVue),
    ] {
        let mut options = NativeVaporSfcCompileOptions::default();
        options.descriptor.version = version;
        options.descriptor.dialect = dialect;
        let compilation = compile_native_vapor_sfc(&arena, source, options);
        assert!(
            matches!(compilation.result(), Err(NativeVaporSfcCompileError::Source(issue)) if issue.kind == NativeSelectedSfcIssueKind::Descriptor)
        );
    }
}

#[test]
fn completed_original_click_file_is_refused_by_vapor_decisions() {
    use vize_l3::decision::vapor::VaporUnsupported;
    for (template, reason) in [(
        r#"<div @click="var unused=$event;"/>"#,
        VaporUnsupported::Binding,
    )] {
        let arena = Allocator::default();
        let source = format!("<template>{template}</template>");
        let compilation = compile_native_vapor_sfc(&arena, &source, Default::default());
        assert!(compilation.observation().issues().is_empty(), "{template}");
        assert!(compilation.observation().admitted().is_some());
        assert!(
            compilation
                .observation()
                .template()
                .unwrap()
                .file()
                .unwrap()
                .is_complete()
        );
        assert!(
            matches!(compilation.result(), Err(NativeVaporSfcCompileError::Vapor(error))
            if error.kind == VaporErrorKind::Unsupported(reason))
        );
    }
}

#[test]
fn original_numeric_for_refusal_keeps_the_actual_rejected_head() {
    use vize_l1::embed::syntax::NativeForRefusal;
    use vize_l2::{file::RejectedFileFor, lang::js::NativeTemplateIssueKind};
    let arena = Allocator::default();
    let source = r#"<template><div v-for="item in 2"/></template>"#;
    let compilation = compile_native_vapor_sfc(&arena, source, Default::default());
    let Err(NativeVaporSfcCompileError::Source(issue)) = compilation.result() else {
        panic!("original unsupported collection must refuse before target output");
    };
    assert!(
        matches!(issue.kind, NativeSelectedSfcIssueKind::Template(native)
        if matches!(native.kind, NativeTemplateIssueKind::For { .. }))
    );
    assert!(compilation.observation().admitted().is_none());
    let original = compilation.observation().template().unwrap();
    let file = original.rejected_file().unwrap();
    let [RejectedFileFor::Syntax(head)] = file.rejected_for_heads() else {
        panic!("retain the actual unsupported original For syntax");
    };
    assert_eq!(head.kind, NativeForRefusal::CollectionShape);
    let syntax = head.operand().syntax();
    assert_eq!(syntax.source().text(), "item in 2");
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
}

#[test]
fn original_fragment_roots_refuse_whole_component_lifecycle_before_output() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native_vapor_sfc_vue_3_6_rc9.json")).unwrap();
    let cases = pack["refusedReferences"].as_array().unwrap();
    assert_eq!(cases.len(), 2);
    for fixture in cases {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let compilation = compile_native_vapor_sfc(&arena, source, Default::default());
        assert!(compilation.observation().admitted().is_some());
        let analysis = build_native_vapor_file_decisions(
            compilation
                .observation()
                .admitted()
                .unwrap()
                .into_template_view(),
        )
        .unwrap();
        assert!(analysis.vapor().unwrap().unsupported().is_empty());
        assert!(analysis.vapor().unwrap().roots().len() > 1);
        assert!(
            matches!(compilation.result(), Err(NativeVaporSfcCompileError::Vapor(error))
            if error.kind == VaporErrorKind::ComponentFragmentLifecycle)
        );
        assert_eq!(
            emit_component::<Recorded>(&analysis, "3.6.0-rc.9").unwrap_err(),
            emit_component::<NoLinks>(&analysis, "3.6.0-rc.9").unwrap_err()
        );
        assert!(vize_l4::targets::vapor::emit_template::<Recorded>(&analysis).is_ok());
    }
}
