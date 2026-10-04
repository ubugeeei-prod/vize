//! Complete genuine primitive setup component modules, maps and refusals.
use vize_atelier_sfc::{
    NativeVaporSetupSfcCompileError, NativeVaporSfcCompileOptions, compile_native_vapor_setup_sfc,
};
use vize_l0::Allocator;
use vize_l3::decision::vapor::{VaporUnsupported, build_native_selected_setup_vapor_decisions};
use vize_l4::{
    module::AssemblyError,
    targets::vapor::{VaporErrorKind, emit_selected_setup_component},
    write::{NoLinks, Recorded},
};

#[test]
fn complete_original_setup_components_and_maps_match_independent_expectations() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/native_vapor_setup_sfc_vue_3_6_rc9.json"
    ))
    .unwrap();
    assert_eq!(pack["version"], "3.6.0-rc.9");
    let mut captures = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let options = NativeVaporSfcCompileOptions {
            filename: pack["filename"].as_str().unwrap(),
            source_map: true,
            ..Default::default()
        };
        let compilation = compile_native_vapor_setup_sfc(&arena, source, options);
        let output = compilation.result().unwrap();
        let observation = compilation.observation();
        assert!(core::ptr::eq(
            observation.original().descriptor().source(),
            source
        ));
        let selected = observation.admitted().unwrap();
        let setup = selected.setup();
        let file = setup.file();
        assert!(file.is_complete());
        assert!(core::ptr::eq(file.artifact().source(), source));
        assert!(core::ptr::eq(
            setup.program().source(),
            setup.source().source()
        ));
        let analysis = build_native_selected_setup_vapor_decisions(setup).unwrap();
        assert!(core::ptr::eq(analysis.owner(), setup.owner()));
        let recorded = emit_selected_setup_component::<Recorded>(&analysis, "3.6.0-rc.9").unwrap();
        let unrecorded = emit_selected_setup_component::<NoLinks>(&analysis, "3.6.0-rc.9").unwrap();
        assert_eq!(recorded.text, unrecorded.text);
        assert_eq!(recorded.helpers, unrecorded.helpers);
        assert_eq!(recorded.text.as_str(), output.code());
        let plain = compile_native_vapor_setup_sfc(
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
        assert_eq!(
            output.code(),
            fixture["code"].as_str().unwrap(),
            "{}",
            fixture["id"]
        );
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        assert_eq!(map, fixture["map"], "{}", fixture["id"]);
        let links = output.document().links();
        let expected = fixture["links"].as_array().unwrap();
        assert_eq!(links.len(), expected.len(), "{}", fixture["id"]);
        for (link, row) in links.iter().zip(expected) {
            assert_eq!(
                u64::from(link.generated.start),
                row["generated"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(link.generated.end),
                row["generatedEnd"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(link.authored.start),
                row["source"].as_u64().unwrap()
            );
            assert_eq!(
                u64::from(link.authored.end),
                row["sourceEnd"].as_u64().unwrap()
            );
            assert_eq!(link.name.as_deref(), row["name"].as_str());
            assert!(link.segment);
            assert!(source.is_char_boundary(link.authored.start as usize));
            assert!(source.is_char_boundary(link.authored.end as usize));
            assert!(
                output
                    .code()
                    .is_char_boundary(link.generated.start as usize)
            );
            assert!(output.code().is_char_boundary(link.generated.end as usize));
        }
        captures.push(serde_json::json!({ "id": fixture["id"], "source": source,
            "code": output.code(), "map": map,
            "roots": analysis.vapor().unwrap().roots().len(),
            "nodes": file.artifact().node_count(), "annotations": setup.type_annotations().count() }));
    }
    assert_eq!(captures.len(), 17);
    if let Some(path) = std::env::var_os("VIZE_NATIVE_VAPOR_SETUP_SFC_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

#[test]
fn unsupported_whole_setup_sources_preserve_the_authentic_observation() {
    for source in [
        "<template>{{1}}</template>",
        "<script setup></script><template><div/></template>",
        "<script setup>/* only comment */</script><template><div/></template>",
        "<script setup>let count</script><template>{{count}}</template>",
        "<script setup>import {ref} from 'vue'; const count=ref(1)</script><template>{{count}}</template>",
        "<script setup>function count(){return 1}</script><template>{{count}}</template>",
        "<script setup>const count={value:1}</script><template>{{count}}</template>",
        "<script setup>const [count]=[1]</script><template>{{count}}</template>",
        "<script setup>const count=-1</script><template>{{count}}</template>",
        "<script>export default {}</script><script setup>let count=1</script><template>{{count}}</template>",
        "<script setup>let count=1</script><template>{{count}}</template><style></style>",
        "<script setup>let count=1</script><template>{{count}}</template><style scoped>.x{color:red}</style>",
        "<script setup src='./x.ts'></script><template>{{1}}</template>",
        "<script setup lang='tsx'>let count=1</script><template>{{count}}</template>",
        "<script setup>let count=1</script><template lang='pug'>div</template>",
        "<script setup>let count=1</script><template>{{count}}</template><i18n>{}</i18n>",
        "<script setup>let count=1</script><template>{{missing}}</template>",
    ] {
        for source_map in [false, true] {
            let arena = Allocator::default();
            let compilation = compile_native_vapor_setup_sfc(
                &arena,
                source,
                NativeVaporSfcCompileOptions {
                    source_map,
                    ..Default::default()
                },
            );
            assert!(
                matches!(
                    compilation.result(),
                    Err(NativeVaporSetupSfcCompileError::Source(_))
                ),
                "{source}: {:?}",
                compilation.result()
            );
            assert!(core::ptr::eq(
                compilation.observation().original().descriptor().source(),
                source
            ));
            assert!(compilation.observation().admitted().is_none());
        }
    }
}

#[test]
fn complete_original_control_and_handler_files_remain_vapor_target_refusals() {
    for (template, reason) in [
        ("{{count+1}}", VaporUnsupported::Expression),
        ("{{\\u0063ount}}", VaporUnsupported::Expression),
        (
            "<div @click='var unused=$event;'/>",
            VaporUnsupported::Binding,
        ),
        ("<div v-for='item in count'/>", VaporUnsupported::Operation),
        (
            "<span>{{count}}</span>",
            VaporUnsupported::NestedInterpolation,
        ),
        ("<search/>", VaporUnsupported::ElementSemantics),
    ] {
        let source = format!("<script setup>let count=1</script><template>{template}</template>");
        let arena = Allocator::default();
        let compilation = compile_native_vapor_setup_sfc(&arena, &source, Default::default());
        assert_eq!(
            compilation.observation().original().issues(),
            [],
            "{source}"
        );
        let selected = compilation.observation().admitted().unwrap();
        assert!(selected.setup().file().is_complete());
        let Err(NativeVaporSetupSfcCompileError::Vapor(error)) = compilation.result() else {
            panic!("target refusal: {source}")
        };
        assert_eq!(error.kind, VaporErrorKind::Unsupported(reason), "{source}");
    }
}

#[test]
fn original_numeric_for_keeps_its_precise_source_refusal_before_vapor() {
    use vize_l1::embed::syntax::NativeForRefusal;
    use vize_l2::file::RejectedFileFor;
    let arena = Allocator::default();
    let source = "<script setup>let count=1</script><template><div v-for='item in 2'/></template>";
    let compilation = compile_native_vapor_setup_sfc(&arena, source, Default::default());
    assert!(matches!(
        compilation.result(),
        Err(NativeVaporSetupSfcCompileError::Source(_))
    ));
    assert!(compilation.observation().admitted().is_none());
    let original = compilation.observation().original().template().unwrap();
    let file = original.file().unwrap();
    assert!(!file.is_complete());
    let [RejectedFileFor::Syntax(head)] = file.rejected_for_heads() else {
        panic!("retain the genuine refused numeric collection")
    };
    assert_eq!(head.kind, NativeForRefusal::CollectionShape);
    let syntax = head.operand().syntax();
    assert_eq!(syntax.source().text(), "item in 2");
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
}

#[test]
fn complete_original_hazardous_strings_are_precise_all_or_nothing_target_refusals() {
    for literal in [
        r#"'a\0b'"#,
        r#"'a\u0000b'"#,
        r#"'a\rb'"#,
        r#"'a\r\nb'"#,
        r#"'a\ud800b'"#,
        r#"'a\udc00b'"#,
    ] {
        for keyword in ["const", "let", "var"] {
            for expression in ["count", literal] {
                for source_map in [false, true] {
                    let source = format!(
                        "<script setup>{keyword} count={literal};</script><template>{{{{{expression}}}}}</template>"
                    );
                    let arena = Allocator::default();
                    let compilation = compile_native_vapor_setup_sfc(
                        &arena,
                        &source,
                        NativeVaporSfcCompileOptions {
                            source_map,
                            ..Default::default()
                        },
                    );
                    assert_eq!(
                        compilation.observation().original().issues(),
                        [],
                        "{source}"
                    );
                    let selected = compilation.observation().admitted().unwrap();
                    assert!(selected.setup().file().is_complete());
                    let Err(NativeVaporSetupSfcCompileError::Vapor(error)) = compilation.result()
                    else {
                        panic!("genuine hazardous value refuses all output")
                    };
                    assert_eq!(
                        error.kind,
                        VaporErrorKind::Unsupported(VaporUnsupported::StringNormalization)
                    );
                }
            }
        }
    }
}

#[test]
fn real_fragment_lifecycle_collisions_and_runtime_version_refuse_complete_sources() {
    for (source, expected) in [
        (
            "<script setup>let count=1</script><template></template>",
            VaporErrorKind::ComponentEmptySetupTemplate,
        ),
        (
            "<script setup>let count=1</script><template><!--before-->{{count}}<!--after--></template>",
            VaporErrorKind::ComponentFragmentLifecycle,
        ),
        (
            "<script setup>const n0=1</script><template>{{n0}}</template>",
            VaporErrorKind::GeneratedBindingCollision,
        ),
        (
            "<script setup>const _setText=1</script><template>{{_setText}}</template>",
            VaporErrorKind::GeneratedBindingCollision,
        ),
        (
            "<script setup>const _template=1</script><template><div/></template>",
            VaporErrorKind::GeneratedBindingCollision,
        ),
    ] {
        let arena = Allocator::default();
        let compilation = compile_native_vapor_setup_sfc(&arena, source, Default::default());
        assert_eq!(
            compilation.observation().original().issues(),
            [],
            "{source}"
        );
        assert!(compilation.observation().admitted().is_some());
        let Err(NativeVaporSetupSfcCompileError::Vapor(error)) = compilation.result() else {
            panic!("complete source target refusal")
        };
        assert_eq!(error.kind, expected);
    }
    let arena = Allocator::default();
    let source = "<script setup>let count=1</script><template>{{count}}</template>";
    let compilation = compile_native_vapor_setup_sfc(
        &arena,
        source,
        NativeVaporSfcCompileOptions {
            runtime_version: "3.5.35",
            source_map: true,
            ..Default::default()
        },
    );
    assert!(compilation.observation().admitted().is_some());
    let Err(NativeVaporSetupSfcCompileError::Vapor(error)) = compilation.result() else {
        panic!("exact pinned runtime only")
    };
    assert_eq!(
        error.kind,
        VaporErrorKind::Assembly(AssemblyError::UnsupportedRuntimeVersion)
    );
}

#[test]
fn original_nested_whitespace_is_a_source_refusal_before_vapor() {
    use vize_l1_to_l2::native_file::NativeSelectedSfcIssueKind;
    use vize_l2::lang::js::NativeTemplateIssueKind;
    let arena = Allocator::default();
    let source = "<script setup>let count=1</script><template><div>a  b</div></template>";
    let compilation = compile_native_vapor_setup_sfc(&arena, source, Default::default());
    assert!(matches!(
        compilation.result(),
        Err(NativeVaporSetupSfcCompileError::Source(_))
    ));
    assert!(compilation.observation().admitted().is_none());
    let original = compilation.observation().original();
    let [issue] = original.issues() else {
        panic!("exact original child refusal")
    };
    assert!(
        matches!(issue.kind, NativeSelectedSfcIssueKind::Template(child)
        if child.kind == NativeTemplateIssueKind::UnsupportedChild)
    );
    assert_eq!(issue.span, vize_l0::Span::new(44, 59));
    assert!(!original.template().unwrap().file().unwrap().is_complete());
    assert!(core::ptr::eq(original.descriptor().source(), source));
}
