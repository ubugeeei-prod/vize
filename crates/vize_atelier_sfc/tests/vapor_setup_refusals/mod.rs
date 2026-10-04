//! Genuine original source and target refusal laws.
use super::*;

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

#[test]
fn complete_original_normalizing_comments_are_precise_setup_target_refusals() {
    use vize_l3::decision::vapor::VaporPart;
    let mut sources = Vec::new();
    for content in ["a&b", "a<b", "a>b", "a\"b", "a'b"] {
        for template in [
            format!("<!--{content}-->"),
            format!("<div><!--{content}--></div>"),
        ] {
            sources.push((
                format!("<script setup>const unused=1</script><template>{template}</template>"),
                format!("<!--{content}-->"),
            ));
        }
    }
    // Preserve the exact authentic fixture that failed in real Chromium.
    sources.push(("<script setup>const unused='import x';</script><template><!--import { template as _template } from 'vue'--></template>".into(), "<!--import { template as _template } from 'vue'-->".into()));
    for (source, expected_comment) in sources {
        for source_map in [false, true] {
            let arena = Allocator::default();
            let compilation = compile_native_vapor_setup_sfc(
                &arena,
                &source,
                NativeVaporSfcCompileOptions {
                    source_map,
                    ..Default::default()
                },
            );
            let observation = compilation.observation();
            assert!(core::ptr::eq(
                observation.original().descriptor().source(),
                source.as_str()
            ));
            assert_eq!(observation.original().issues(), [], "{source}");
            let selected = observation.admitted().unwrap();
            let setup = selected.setup();
            assert!(setup.file().is_complete());
            let analysis = build_native_selected_setup_vapor_decisions(setup).unwrap();
            let facts = analysis.vapor().unwrap();
            assert!(facts.unsupported().is_empty(), "genuine L3 admission");
            let (node, span) = facts
                .roots()
                .iter()
                .flat_map(|root| facts.parts(root).unwrap())
                .find_map(|part| match part {
                    VaporPart::Comment { node, comment } => Some((*node, comment.span)),
                    _ => None,
                })
                .unwrap();
            let Err(NativeVaporSetupSfcCompileError::Vapor(error)) = compilation.result() else {
                panic!("normalizing original comment refuses all output: {source}")
            };
            assert_eq!(error.kind, VaporErrorKind::CommentNormalization);
            assert_eq!(error.node, Some(node));
            assert_eq!(error.span, span);
            let original = source.get(span.start as usize..span.end as usize).unwrap();
            assert_eq!(original, expected_comment);
        }
    }
}
