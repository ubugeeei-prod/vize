use super::*;
use crate::container::{ContainerFormat, Vue};
use crate::embed::EmbedSource;
use vize_l0::{SourceFrameError, String};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn has_issue(source: &str, code: DescriptorIssueCode) {
    let allocator = Allocator::default();
    let observed = Vue.observe_descriptor(&allocator, source, options());
    let rejected = observed.admitted().expect_err(source);
    assert!(
        rejected.issues().iter().any(|issue| issue.code == code),
        "{source}: {rejected:?}"
    );
}

#[test]
fn reversed_roles_keep_original_indices_whole_source_and_raw_unicode() {
    let source = "<!-- 日本語 -->\r\n<template>{{ x }} &amp;</template>\r\n<script setup lang='ts'>const x = '&amp;日本語';\r\n</script>\r\n<script lang=ts>export const y = 1;</script>tail";
    let allocator = Allocator::default();
    let observation = Vue.observe_descriptor(&allocator, source, options());
    let view = observation.admitted().expect("actual descriptor");
    let ordinary = view.ordinary().expect("ordinary");
    let setup = view.setup().expect("setup");
    let template = view.template().expect("template");
    assert_eq!(
        (
            ordinary.container_index(),
            setup.container_index(),
            template.container_index()
        ),
        (2, 1, 0)
    );
    assert_eq!(
        (ordinary.role(), setup.role()),
        (ScriptRole::Ordinary, ScriptRole::Setup)
    );
    assert_eq!(
        (ordinary.lang(), setup.lang(), view.template_lang()),
        (Lang::Ts, Lang::Ts, Lang::Ts)
    );
    assert_eq!(view.source().as_ptr(), source.as_ptr());
    assert_eq!(ordinary.source().as_ptr(), source.as_ptr());
    assert_eq!(template.source().as_ptr(), source.as_ptr());
    for (index, block) in [
        (2, ordinary.block()),
        (1, setup.block()),
        (0, template.block()),
    ] {
        assert_eq!(block.root_source().as_ptr(), source.as_ptr());
        assert_eq!(block.span(), observation.container().blocks[index].content);
        let authored =
            EmbedSource::authored(block.root_source(), block.span()).expect("whole-root authored");
        assert_eq!(authored.text().as_ptr(), block.source().as_ptr());
        assert!(authored.decode_map().is_none());
        assert_eq!(authored.span(), block.span());
    }
    assert_eq!(setup.block().source(), "const x = '&amp;日本語';\r\n");
    assert_eq!(
        observation.container().blocks[1]
            .attr("lang")
            .unwrap()
            .value,
        Some("ts")
    );
    assert_eq!(observation.options(), options());
    assert_eq!(observation.source(), source);
}

#[test]
fn setup_only_is_legal_and_template_inherits_its_real_ts_profile() {
    for source in [
        "<script setup lang=ts>const x = 1</script><template>{{ x }}</template>",
        "<template>{{ x }}</template><script lang='ts' setup>const x = 1</script>",
    ] {
        let allocator = Allocator::default();
        let observation = Vue.observe_descriptor(&allocator, source, options());
        let view = observation.admitted().expect("setup only legal");
        assert!(view.ordinary().is_none());
        assert_eq!(view.setup().unwrap().role(), ScriptRole::Setup);
        assert_eq!(view.template_lang(), Lang::Ts);
    }
}

#[test]
fn no_scripts_defaults_template_to_js_without_source_inference() {
    let allocator = Allocator::default();
    let observation = Vue.observe_descriptor(
        &allocator,
        "<template lang=html>{{ ts }}</template>",
        options(),
    );
    let view = observation.admitted().expect("ordinary HTML template");
    assert_eq!(view.template_lang(), Lang::Js);
    assert!(view.setup().is_none());
    assert!(view.ordinary().is_none());
}

#[test]
fn foreign_equal_block_cannot_become_this_root_slice() {
    let source = String::from("<script>const x = 1</script>");
    let allocator = Allocator::default();
    let observation = Vue.observe_descriptor(&allocator, &source, options());
    let block = observation.admitted().unwrap().ordinary().unwrap().block();
    let foreign = String::from(block.source());
    assert_eq!(foreign.as_str(), block.source());
    assert_ne!(foreign.as_ptr(), block.source().as_ptr());
    assert_eq!(
        observation.root().unwrap().block(&foreign, block.start()),
        Err(SourceFrameError::BlockNotRootSlice)
    );
}

#[test]
fn original_capture_and_all_diagnostics_are_retained_byte_for_byte() {
    for source in [
        "lead<template>x</template>between<script setup lang=ts>x</script>tail",
        "<template>x</template><template>y",
        "<template>{{ `value ${name}` }}</template>",
        "<script lang='ts",
        "<style scoped>.x {}</style><i18n lang=json>{\"x\":1}</i18n>",
    ] {
        let allocator = Allocator::default();
        let capture = Vue.split(&allocator, source);
        let observation = Vue.observe_descriptor(&allocator, source, options());
        let actual = observation.container();
        assert_eq!(actual.source.as_ptr(), capture.source.as_ptr());
        assert_eq!(&*actual.errors, &*capture.errors);
        assert_eq!(actual.blocks.len(), capture.blocks.len());
        for (left, right) in actual.blocks.iter().zip(capture.blocks.iter()) {
            assert_eq!(left.name.as_ptr(), right.name.as_ptr());
            assert_eq!(
                (left.open_tag, left.content, left.close_tag),
                (right.open_tag, right.content, right.close_tag)
            );
            assert_eq!(&*left.attrs, &*right.attrs);
            for (left, right) in left.attrs.iter().zip(right.attrs.iter()) {
                assert_eq!(left.name.as_ptr(), right.name.as_ptr());
                assert_eq!(left.value.map(str::as_ptr), right.value.map(str::as_ptr));
            }
        }
        if !capture.errors.is_empty() {
            assert!(observation.admitted().is_err());
        }
    }
}

#[test]
fn language_attributes_and_roles_refuse_ambiguous_original_evidence() {
    for (source, code) in [
        (
            "<script lang=ts lang=js></script>",
            DescriptorIssueCode::DuplicateAttribute,
        ),
        (
            "<script LANG=ts lang=ts></script>",
            DescriptorIssueCode::DuplicateAttribute,
        ),
        (
            "<script setup setup></script>",
            DescriptorIssueCode::DuplicateAttribute,
        ),
        (
            "<script lang='ts'setup></script>",
            DescriptorIssueCode::AmbiguousAttribute,
        ),
        (
            "<script setup=''></script>",
            DescriptorIssueCode::UnsupportedSetupValue,
        ),
        (
            "<script src='file.ts'></script>",
            DescriptorIssueCode::ExternalSource,
        ),
        (
            "<template src=template.html></template>",
            DescriptorIssueCode::ExternalSource,
        ),
        (
            "<script lang='t&#115;'></script>",
            DescriptorIssueCode::EncodedLanguage,
        ),
        (
            "<script lang=tsx></script>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
        (
            "<script lang=javascript></script>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
        (
            "<script lang></script>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
        (
            "<template lang=pug></template>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
        (
            "<script generic=T lang=ts></script>",
            DescriptorIssueCode::UnsupportedAttribute,
        ),
        (
            "<script></script><script setup lang=ts></script>",
            DescriptorIssueCode::LanguageMismatch,
        ),
        (
            "<script setup lang=ts></script><script></script>",
            DescriptorIssueCode::LanguageMismatch,
        ),
        (
            "<script></script><script></script>",
            DescriptorIssueCode::DuplicateRole,
        ),
        (
            "<script setup></script><script setup></script>",
            DescriptorIssueCode::DuplicateRole,
        ),
        (
            "<template></template><template></template>",
            DescriptorIssueCode::DuplicateRole,
        ),
        (
            "<SCRIPT></SCRIPT>",
            DescriptorIssueCode::UnsupportedBlockSpelling,
        ),
        ("<script/>", DescriptorIssueCode::UnsupportedBoundary),
        (
            "<script>const x=1",
            DescriptorIssueCode::UnsupportedBoundary,
        ),
        (
            "<template>{{ `x ${y}` }}</template>",
            DescriptorIssueCode::UnsupportedBoundary,
        ),
        (
            "<style scoped>.x{}</style>",
            DescriptorIssueCode::UnsupportedBlock,
        ),
        ("<custom>x</custom>", DescriptorIssueCode::UnsupportedBlock),
    ] {
        has_issue(source, code);
    }
}

#[test]
fn explicit_unsupported_dialect_and_options_stay_retained() {
    let allocator = Allocator::default();
    for (selected, code) in [
        (
            DescriptorOptions {
                version: VueVersion::V2_7,
                ..options()
            },
            DescriptorIssueCode::UnsupportedVersion,
        ),
        (
            DescriptorOptions {
                dialect: VueDialect::PetiteVue,
                ..options()
            },
            DescriptorIssueCode::UnsupportedDialect,
        ),
        (
            DescriptorOptions {
                template: SurfaceParseOptions {
                    experimental_in_tag_comments: true,
                },
                ..options()
            },
            DescriptorIssueCode::UnsupportedOptions,
        ),
    ] {
        let observation =
            Vue.observe_descriptor(&allocator, "<script setup lang=ts>x</script>", selected);
        assert_eq!(observation.options(), selected);
        assert_eq!(observation.container().blocks.len(), 1);
        assert!(
            observation
                .admitted()
                .unwrap_err()
                .issues()
                .iter()
                .any(|issue| issue.code == code)
        );
    }
}

#[test]
fn malformed_open_and_close_keep_typed_splitter_refusals() {
    for source in [
        "<script lang='ts",
        "<script>x",
        "<template>{{ x </template>",
    ] {
        let allocator = Allocator::default();
        let observation = Vue.observe_descriptor(&allocator, source, options());
        let refusal = observation
            .admitted()
            .expect_err("uncertain original parser evidence");
        assert!(!refusal.errors().is_empty());
        assert_eq!(refusal.errors(), &*observation.container().errors);
    }
}

#[test]
fn admission_requires_an_actual_component_block_without_dropping_empty_blocks() {
    for source in ["", "hello", "<!-- no component block -->"] {
        has_issue(source, DescriptorIssueCode::MissingComponentBlock);
    }
    for source in [
        "<template></template>",
        "<script></script>",
        "<script setup></script>",
    ] {
        let allocator = Allocator::default();
        let owner = Vue.observe_descriptor(&allocator, source, options());
        assert_eq!(owner.container().blocks.len(), 1);
        assert!(
            owner.admitted().is_ok(),
            "structural original selection: {source}"
        );
    }
}
