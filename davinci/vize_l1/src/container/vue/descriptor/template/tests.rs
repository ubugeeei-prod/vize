use super::*;
use crate::SurfaceParseOptions;
use crate::container::vue::{DescriptorOptions, Vue};
use vize_l0::Allocator;
use vize_l0::config::{VueDialect, VueVersion};

mod custody;
mod refusals;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn utf16(source: &str, offset: u32) -> (usize, usize) {
    let prefix = source.get(..offset as usize).unwrap();
    (
        prefix.bytes().filter(|byte| *byte == b'\n').count(),
        prefix.rsplit('\n').next().unwrap().encode_utf16().count(),
    )
}

#[test]
fn original_3471_outer_names_keep_complete_source_script_role_and_order() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/vize_maestro/tests/fixtures/native-linked-history-3471.vue"
    ));
    assert_eq!(
        source,
        "<script setup lang=\"ts\">\nconst count = 1\n</script>\n\n<template>\n  <div class=\"a\">\n    <div class=\"b\">{{ count }}</div>\n  </div>\n  <div class=\"c\" />\n</template>\n"
    );
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let admitted = owner.admitted().unwrap();
    let template = admitted.template().unwrap();
    let names = template.frame_names().unwrap();
    assert_eq!(names.opening(), Span::new(53, 61));
    assert_eq!(names.closing(), Span::new(149, 157));
    assert_eq!(utf16(source, names.opening().start), (4, 1));
    assert_eq!(utf16(source, names.opening().end), (4, 9));
    assert_eq!(utf16(source, names.closing().start), (9, 2));
    assert_eq!(utf16(source, names.closing().end), (9, 10));
    assert_eq!(names.source_block().source().as_ptr(), source.as_ptr());
    assert_eq!(names.container_index(), 1);
    assert!(names.accepts(&template));
    assert!(names.accepts(&names.template()));
    assert_eq!(admitted.setup().unwrap().container_index(), 0);
    assert_eq!(admitted.setup().unwrap().lang(), crate::embed::Lang::Ts);
    assert!(!template.block().contains_block_span(names.opening()));
    assert!(!template.block().contains_block_span(names.closing()));
}

#[test]
fn original_empty_template_still_has_two_nonempty_frame_names() {
    let source = "<template></template>";
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let template = owner.admitted().unwrap().template().unwrap();
    let names = template.frame_names().unwrap();
    assert_eq!(template.block().span(), Span::new(10, 10));
    assert_eq!(names.opening(), Span::new(1, 9));
    assert_eq!(names.closing(), Span::new(12, 20));
    assert_eq!(names.source_block().span(), Span::new(0, 21));
}

#[test]
fn original_unicode_crlf_nonzero_template_uses_whole_root_utf8_and_utf16() {
    let source = "<!-- 😀日本語 -->\r\n<script setup lang='ts'>const x=1</script>\r\n<template lang=html>\r\n<p>😀</p>\r\n</template \t>\r\n<style>.x{}</style>";
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let admitted = owner.admitted().unwrap();
    let template = admitted.template().unwrap();
    let names = template.frame_names().unwrap();
    assert_eq!(names.opening(), Span::new(69, 77));
    assert_eq!(names.closing(), Span::new(105, 113));
    assert_eq!(utf16(source, names.opening().start), (2, 1));
    assert_eq!(utf16(source, names.opening().end), (2, 9));
    assert_eq!(utf16(source, names.closing().start), (4, 2));
    assert_eq!(utf16(source, names.closing().end), (4, 10));
    assert_eq!(template.block().source(), "\r\n<p>😀</p>\r\n");
    assert_eq!(names.container_index(), 1);
    assert_eq!(names.source_block().start(), 0);
    assert_eq!(names.source_block().source().as_ptr(), source.as_ptr());
    assert!(names.source_block().contains_block_span(names.opening()));
    assert!(!template.block().contains_block_span(names.opening()));
    assert_eq!(admitted.setup().unwrap().lang(), crate::embed::Lang::Ts);
    assert_eq!(admitted.styles().next().unwrap().container_index(), 2);
}

#[test]
fn only_original_depth_zero_close_owns_a_nested_template_frame() {
    let source = "<template><template #x>nested</template><template/>after</template>";
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let template = owner.admitted().unwrap().template().unwrap();
    let names = template.frame_names().unwrap();
    assert_eq!(names.opening(), Span::new(1, 9));
    assert_eq!(names.closing(), Span::new(58, 66));
    assert_eq!(
        template.block().source(),
        "<template #x>nested</template><template/>after"
    );
    assert_eq!(owner.container().blocks.len(), 1);
}

#[test]
fn original_comment_attribute_interpolation_and_raw_closers_never_supply_the_frame() {
    let source = "<template><!-- </template> --><p x='</template>'>x</p>{{ '</template>' }}<textarea></template></textarea></template>";
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let template = owner.admitted().unwrap().template().unwrap();
    let names = template.frame_names().unwrap();
    assert_eq!(names.closing(), Span::new(107, 115));
    assert_eq!(owner.container().blocks.len(), 1);
    assert!(template.block().source().ends_with("</textarea>"));
}

#[test]
fn original_ascii_closing_case_and_post_name_gap_are_retained_without_normalization() {
    let arena = Allocator::default();
    for source in ["<template></TeMPLATE >", "<template></template \t\r\n>"] {
        let owner = Vue.observe_descriptor(&arena, source, options());
        let names = owner
            .admitted()
            .unwrap()
            .template()
            .unwrap()
            .frame_names()
            .unwrap();
        assert_eq!(names.opening(), Span::new(1, 9));
        assert_eq!(names.closing(), Span::new(12, 20));
        let closing = source.get(12..20).unwrap();
        assert_eq!(
            closing,
            if source.contains("TeMPLATE") {
                "TeMPLATE"
            } else {
                "template"
            }
        );
        assert_eq!(source.get(1..9), Some("template"));
    }
}
