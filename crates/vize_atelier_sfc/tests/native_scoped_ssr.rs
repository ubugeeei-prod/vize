use vize_atelier_sfc::{
    NativeScopedSsrSfcCompileError as Error, NativeScopedSsrSfcCompileOptions as Options,
    NativeScopedSsrSfcOutput, NativeSfcCompileError, NativeSsrSfcCompileOptions,
    compile_native_scoped_ssr_sfc, compile_native_ssr_sfc,
};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::NativeSelectedSfcIssueKind as LowerKind;
use vize_l2::lang::js::NativeScopedTemplateIssueKind;
use vize_l3::decision::ssr::{
    NativeSsrBuildError, SsrPart, build_native_scoped_ssr_file_decisions,
    build_native_ssr_file_decisions,
};
use vize_l4::module::AssemblyError;

fn pack() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/native-scoped-ssr-vue-3.5.35.json")).unwrap()
}
fn links(document: &vize_l4::write::EmitDocument) -> Vec<serde_json::Value> {
    document.links().iter().map(|link| serde_json::json!({"authored":{"start":link.authored.start,"end":link.authored.end},"generated":{"start":link.generated.start,"end":link.generated.end},"name":link.name.as_deref(),"segment":link.segment})).collect()
}
fn original_css_bytes_and_generated_scope(
    source: &str,
    original: &str,
    span: vize_l0::Span,
    output: &NativeScopedSsrSfcOutput,
) {
    let document = output.css_document().unwrap();
    let mut cursor = 0;
    let mut original_cursor = span.start;
    let mut authored = String::new();
    let mut gaps = Vec::new();
    for link in document.links() {
        assert_eq!(link.authored.start, original_cursor);
        assert!(link.authored.end >= original_cursor && link.authored.end <= span.end);
        original_cursor = link.authored.end;
        let start = link.generated.start as usize;
        let end = link.generated.end as usize;
        if cursor < start {
            gaps.push(&document.as_str()[cursor..start]);
        }
        let text = &document.as_str()[start..end];
        assert_eq!(link.authored.slice(source), text);
        authored.push_str(text);
        cursor = end;
    }
    assert_eq!(cursor, document.as_str().len());
    assert_eq!(gaps, [format!("[{}]", output.scope_id())]);
    assert_eq!(authored, original);
    assert_eq!(original_cursor, span.end);
}

#[test]
fn three_whole_original_scoped_ssr_modules_css_maps_and_file_custody_match() {
    let pack = pack();
    assert_eq!(pack["fixtures"].as_array().unwrap().len(), 3);
    let mut captures = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let filename = fixture["filename"].as_str().unwrap();
        let options = Options {
            filename,
            scope_id: fixture["explicitScopeId"].as_str(),
            source_map: true,
            ..Options::default()
        };
        let compilation = compile_native_scoped_ssr_sfc(&arena, source, options);
        let tokens = compilation
            .observation()
            .style_syntax()
            .unwrap()
            .rule()
            .unwrap()
            .prelude()
            .as_ptr();
        let moved = core::hint::black_box(compilation);
        let observation = moved.observation();
        assert!(
            observation.original().issues().is_empty(),
            "{}: {:?}",
            fixture["id"],
            observation.original().issues()
        );
        // The normally moved owner's original parser token storage survives.
        assert_eq!(
            observation
                .style_syntax()
                .unwrap()
                .rule()
                .unwrap()
                .prelude()
                .as_ptr(),
            tokens
        );
        assert!(core::ptr::eq(
            observation.original().descriptor().source(),
            source
        ));
        assert!(observation.original().admitted().is_none());
        let view = observation.admitted().unwrap().into_scoped_template_view();
        let file = view.file();
        assert!(core::ptr::eq(
            file,
            observation.original().template().unwrap().file().unwrap()
        ));
        assert!(file.is_complete());
        assert!(file.units().is_empty());
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert!(core::ptr::eq(
            view.style_syntax(),
            observation.style_syntax().unwrap()
        ));
        let analysis = build_native_scoped_ssr_file_decisions(view).unwrap();
        assert!(core::ptr::eq(analysis.file(), file));
        assert!(core::ptr::eq(
            analysis.scoped().style_syntax(),
            observation.style_syntax().unwrap()
        ));
        assert!(analysis.ssr().unwrap().unsupported().is_empty());
        let openings = analysis
            .ssr()
            .unwrap()
            .parts()
            .iter()
            .filter(|part| matches!(part, SsrPart::Open { .. }))
            .count();
        let output = moved
            .result()
            .unwrap_or_else(|error| panic!("{}: {error:?}", fixture["id"]));
        assert_eq!(output.code(), fixture["code"].as_str().unwrap());
        assert_eq!(output.css(), fixture["css"].as_str());
        assert_eq!(output.scope_id(), fixture["scopeId"].as_str().unwrap());
        assert_eq!(
            output.code().matches(output.scope_id()).count(),
            openings + 1
        );
        for (at, scope) in output.code().match_indices(output.scope_id()) {
            let end = at + scope.len();
            assert!(
                output
                    .document()
                    .links()
                    .iter()
                    .all(|link| link.generated.end as usize <= at
                        || link.generated.start as usize >= end)
            );
        }
        original_css_bytes_and_generated_scope(
            source,
            observation.style_syntax().unwrap().source().source(),
            observation.style_syntax().unwrap().source().span(),
            output,
        );
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        let css_map: serde_json::Value =
            serde_json::from_str(output.css_source_map().unwrap()).unwrap();
        assert_eq!(map, fixture["map"]);
        assert_eq!(css_map, fixture["cssMap"]);
        assert_eq!(output.source_map(), fixture["mapText"].as_str());
        assert_eq!(output.css_source_map(), fixture["cssMapText"].as_str());
        for map in [&map, &css_map] {
            assert_eq!(map["sourcesContent"], serde_json::json!([source]));
            assert_eq!(map["sources"], serde_json::json!([filename]));
            assert!(!map["mappings"].as_str().unwrap().is_empty());
        }
        let unrecorded = compile_native_scoped_ssr_sfc(
            &arena,
            source,
            Options {
                source_map: false,
                ..options
            },
        );
        let unrecorded = unrecorded.result().unwrap();
        assert_eq!(unrecorded.code(), output.code());
        assert_eq!(unrecorded.css(), output.css());
        assert_eq!(unrecorded.scope_id(), output.scope_id());
        assert!(unrecorded.source_map().is_none());
        assert!(unrecorded.css_source_map().is_none());
        assert!(unrecorded.document().links().is_empty());
        assert!(unrecorded.css_document().unwrap().links().is_empty());
        captures.push(serde_json::json!({"id":fixture["id"],"source":source,"filename":filename,"code":output.code(),"css":output.css(),"scopeId":output.scope_id(),"map":map,"cssMap":css_map,"mapText":output.source_map().unwrap(),"cssMapText":output.css_source_map().unwrap(),"links":links(output.document()),"cssLinks":links(output.css_document().unwrap())}));
        let (original, result) = moved.into_parts();
        assert!(result.is_ok());
        assert_eq!(
            original
                .style_syntax()
                .unwrap()
                .rule()
                .unwrap()
                .prelude()
                .as_ptr(),
            tokens
        );
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SCOPED_SSR_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

#[test]
fn original_style_refusals_do_not_grant_partial_scoped_product() {
    for source in [
        "<template><p/></template>",
        "<template><p/></template><style>.a:empty{}</style>",
        "<template><p/></template><style scoped module>.a:empty{}</style>",
        "<template><p/></template><style scoped lang=scss>.a:empty{}</style>",
        "<template><p/></template><style scoped src='outside.css'></style>",
        "<template><p/></template><style scoped>.a:empty{}</style><style scoped>.b:empty{}</style>",
        "<script setup>const value=1;</script><template><p/></template><style scoped>.a:empty{}</style>",
        "<template><p class=a/></template><style scoped>.a:empty{}</style>",
        "<template><p style='color:red'/></template><style scoped>.a:empty{}</style>",
        "<template><search/></template><style scoped>.a:empty{}</style>",
        "<template><p/></template><style scoped>.a:hover{}</style>",
        "<template><p/></template><style scoped>.a:empty{content:'v-bind(tone)'}</style>",
    ] {
        let arena = Allocator::default();
        let compilation = compile_native_scoped_ssr_sfc(&arena, source, Options::default());
        assert!(compilation.result().is_err(), "{source}");
        assert!(core::ptr::eq(
            compilation.observation().original().descriptor().source(),
            source
        ));
    }
    let arena = Allocator::default();
    let source = "<template><p/></template><style scoped>.a:empty{color:red}</style>";
    let compilation = compile_native_scoped_ssr_sfc(&arena, source, Options::default());
    let lower = compilation.observation().original();
    assert!(matches!(
        build_native_ssr_file_decisions(lower.template().unwrap().view().unwrap()),
        Err(NativeSsrBuildError::StyledSource { .. })
    ));
    let old = compile_native_ssr_sfc(&arena, source, NativeSsrSfcCompileOptions::default());
    assert!(old.result().is_err());
    assert_eq!(old.observation().issues()[0].kind, LowerKind::Style);
    let invalid = compile_native_scoped_ssr_sfc(
        &arena,
        source,
        Options {
            scope_id: Some("data-v-x]"),
            ..Options::default()
        },
    );
    assert!(matches!(
        invalid.result(),
        Err(Error::Style(NativeSfcCompileError::Assembly(
            AssemblyError::InvalidScopeId
        )))
    ));
    let runtime = compile_native_scoped_ssr_sfc(
        &arena,
        source,
        Options {
            runtime_version: "99.0.0",
            ..Options::default()
        },
    );
    assert!(matches!(
        runtime.result(),
        Err(Error::Assembly(AssemblyError::UnsupportedRuntimeVersion))
    ));
    let raw = compile_native_scoped_ssr_sfc(
        &arena,
        "<template><p/></template><style>.a:empty{}</style>",
        Options::default(),
    );
    assert!(
        matches!(raw.result(), Err(Error::Lowering(issue)) if issue.kind == LowerKind::ScopedStyle(NativeScopedTemplateIssueKind::StyleProfile))
    );
}

#[test]
fn scoped_ssr_trim_changes_only_outer_css_and_keeps_same_scope_in_all_outputs() {
    let source = "<!-- 雪🌸 -->\r\n<template><p/></template><style scoped> \r\n.雪:empty \t{ content:'雪🌸'; color:red; } \r\n</style>";
    let arena = Allocator::default();
    let options = Options {
        filename: "Scoped雪🌸.vue",
        scope_id: Some("data-v-trim"),
        source_map: true,
        ..Options::default()
    };
    let raw = compile_native_scoped_ssr_sfc(&arena, source, options);
    let trimmed = compile_native_scoped_ssr_sfc(
        &arena,
        source,
        Options {
            style_trim: true,
            ..options
        },
    );
    let raw = raw.result().unwrap();
    let trimmed = trimmed.result().unwrap();
    assert_eq!(trimmed.css(), raw.css().map(str::trim));
    assert_eq!(trimmed.code(), raw.code());
    assert_eq!(trimmed.scope_id(), raw.scope_id());
    for link in trimmed.css_document().unwrap().links() {
        assert_eq!(
            link.authored.slice(source),
            &trimmed.css().unwrap()[link.generated.start as usize..link.generated.end as usize]
        );
    }
}
