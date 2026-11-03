use super::*;
use crate::container::{ContainerFormat, Vue};
use crate::embed::EmbedSource;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

#[test]
fn opaque_styles_keep_every_original_block_attribute_and_source_profile() {
    let source = "<!-- 雪 -->\r\n<style lang='scss' scoped module=theme>/* &amp;雪 */\r\n.x { color: v-bind(color) }</style>\r\n<template><p/></template><style module=''>\r\n.y{color:red}</style><script setup lang=ts>const x=1</script><style></style>tail";
    let arena = Allocator::default();
    let observation = Vue.observe_descriptor(&arena, source, options());
    let descriptor = observation.admitted().unwrap();
    let mut styles = descriptor.styles();
    assert_eq!(styles.len(), 3);
    assert_eq!(descriptor.template().unwrap().container_index(), 1);
    assert_eq!(descriptor.setup().unwrap().container_index(), 3);
    assert_eq!(descriptor.template_lang(), Lang::Ts);
    for index in [0, 2, 4] {
        let style = styles.next().unwrap();
        let original = &observation.container().blocks[index];
        assert_eq!(style.container_index(), index);
        assert!(core::ptr::eq(style.source(), source));
        assert!(core::ptr::eq(style.original_block(), original));
        assert!(core::ptr::eq(style.attrs(), original.attrs.as_slice()));
        assert_eq!(style.block().span(), original.content);
        assert!(core::ptr::eq(style.block().root_source(), source));
        assert!(core::ptr::eq(
            style.block().source(),
            &source[original.content.start as usize..original.content.end as usize]
        ));
        let raw = EmbedSource::authored(source, style.block().span()).unwrap();
        assert!(raw.decode_map().is_none());
        assert!(core::ptr::eq(raw.text(), style.block().source()));
    }
    assert!(styles.next().is_none());
    let first = descriptor.styles().next().unwrap();
    assert_eq!(
        first.block().source(),
        "/* &amp;雪 */\r\n.x { color: v-bind(color) }"
    );
    assert_eq!(first.attrs().len(), 3);
    assert_eq!(first.attrs()[0].name, "lang");
    assert_eq!(first.attrs()[1].name, "scoped");
    assert_eq!(first.attrs()[2].name, "module");
    assert_eq!(first.attrs()[0].value, Some("scss"));
    assert_eq!(first.attrs()[1].value, None);
    assert_eq!(first.attrs()[2].value, Some("theme"));
    assert_eq!(
        descriptor.styles().nth(1).unwrap().attrs()[0].value,
        Some("")
    );

    let capture = Vue.split(&arena, source);
    assert_eq!(capture.source, observation.container().source);
    assert_eq!(capture.errors, observation.container().errors);
    for (left, right) in capture.blocks.iter().zip(&observation.container().blocks) {
        assert_eq!(left.name, right.name);
        assert_eq!(left.open_tag, right.open_tag);
        assert_eq!(left.content, right.content);
        assert_eq!(left.close_tag, right.close_tag);
        assert_eq!(left.attrs, right.attrs);
    }
    assert_eq!(capture.blocks.len(), observation.container().blocks.len());
}

#[test]
fn inline_style_profiles_are_source_data_and_do_not_select_script_language() {
    for profile in [
        "",
        " lang=css",
        " lang=scss",
        " lang=less scoped='false'",
        " module",
        " module='theme'",
        " scoped module=''",
    ] {
        let source = alloc::format!("<style{profile}>raw &amp; original</style><template/>");
        let arena = Allocator::default();
        // The template's original self-closing boundary is independently refused.
        let owner = Vue.observe_descriptor(&arena, &source, options());
        assert_eq!(owner.issues().len(), 1);
        assert_eq!(
            owner.issues()[0].code,
            DescriptorIssueCode::UnsupportedBoundary
        );
        assert_eq!(owner.styles.len(), 1);

        let source = source.replace("<template/>", "<template></template>");
        let owner = Vue.observe_descriptor(&arena, &source, options());
        let view = owner.admitted().unwrap();
        assert_eq!(view.styles().len(), 1);
        assert_eq!(view.template_lang(), Lang::Js);
        assert_eq!(
            view.styles().next().unwrap().block().source(),
            "raw &amp; original"
        );
        assert!(view.ordinary().is_none());
        assert!(view.setup().is_none());
    }
}

#[test]
fn style_refusals_keep_complete_original_content_attributes_and_diagnostics() {
    for (style, code) in [
        (
            "<style src='missing.css'>original</style>",
            DescriptorIssueCode::ExternalSource,
        ),
        (
            "<style lang=css lang=less>original</style>",
            DescriptorIssueCode::DuplicateAttribute,
        ),
        (
            "<style scoped SCOPED>original</style>",
            DescriptorIssueCode::DuplicateAttribute,
        ),
        (
            "<style module module>original</style>",
            DescriptorIssueCode::DuplicateAttribute,
        ),
        (
            "<style lang='css'scoped>original</style>",
            DescriptorIssueCode::AmbiguousAttribute,
        ),
        (
            "<style lang='c&#115;s'>original</style>",
            DescriptorIssueCode::EncodedLanguage,
        ),
        (
            "<style lang>original</style>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
        (
            "<style lang=''>original</style>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
        (
            "<style custom>original</style>",
            DescriptorIssueCode::UnsupportedAttribute,
        ),
        (
            "<STYLE>original</STYLE>",
            DescriptorIssueCode::UnsupportedBlockSpelling,
        ),
        ("<style/>", DescriptorIssueCode::UnsupportedBoundary),
        ("<style>original", DescriptorIssueCode::UnsupportedBoundary),
        (
            "<custom>original</custom>",
            DescriptorIssueCode::UnsupportedBlock,
        ),
    ] {
        let source = alloc::format!("<template></template>{style}");
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, &source, options());
        let refusal = owner.admitted().unwrap_err();
        assert!(
            refusal
                .issues()
                .iter()
                .any(|issue| issue.code == code && issue.container_index == Some(1))
        );
        assert_eq!(owner.container().blocks.len(), 2);
        assert!(core::ptr::eq(owner.source(), source.as_str()));
        let capture = Vue.split(&arena, &source);
        assert_eq!(owner.container().errors, capture.errors);
        assert_eq!(owner.container().blocks[1].attrs, capture.blocks[1].attrs);
        assert_eq!(
            owner.container().blocks[1].content,
            capture.blocks[1].content
        );
    }
}

#[test]
fn style_alone_never_satisfies_component_cardinality() {
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, "<style>.x{}</style>", options());
    assert_eq!(owner.styles.len(), 1);
    assert_eq!(owner.issues().len(), 1);
    assert_eq!(
        owner.admitted().unwrap_err().issues()[0].code,
        DescriptorIssueCode::MissingComponentBlock
    );
    assert_eq!(owner.issues()[0].container_index, None);
}
