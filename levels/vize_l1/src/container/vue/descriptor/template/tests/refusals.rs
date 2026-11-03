use super::*;
use crate::container::vue::{DescriptorIssueCode as Code, ScriptRole};
use crate::container::{BlockAttr, ContainerErrorCode, ContainerFormat};
use crate::embed::Lang;

#[test]
fn every_original_public_capture_field_keeps_its_authored_golden_value() {
    let source = "lead<template lang='html'>x</template >between<script setup lang=ts>const x=1</script><style scoped>.x{}</style>tail";
    let arena = Allocator::default();
    let capture = Vue.split(&arena, source);
    let owner = Vue.observe_descriptor(&arena, source, options());
    let observed = owner.container();
    assert_eq!(capture.source.as_ptr(), source.as_ptr());
    assert_eq!(observed.source.as_ptr(), source.as_ptr());
    assert!(capture.errors.is_empty());
    assert!(observed.errors.is_empty());
    assert_eq!(capture.blocks.len(), 3);
    let expected = [
        (
            "template",
            Span::new(4, 26),
            Span::new(26, 27),
            Span::new(27, 39),
        ),
        (
            "script",
            Span::new(46, 68),
            Span::new(68, 77),
            Span::new(77, 86),
        ),
        (
            "style",
            Span::new(86, 100),
            Span::new(100, 104),
            Span::new(104, 112),
        ),
    ];
    let attrs: [&[BlockAttr<'_>]; 3] = [
        &[BlockAttr {
            name: "lang",
            value: Some("html"),
            span: Span::new(14, 25),
        }],
        &[
            BlockAttr {
                name: "setup",
                value: None,
                span: Span::new(54, 60),
            },
            BlockAttr {
                name: "lang",
                value: Some("ts"),
                span: Span::new(60, 67),
            },
        ],
        &[BlockAttr {
            name: "scoped",
            value: None,
            span: Span::new(93, 99),
        }],
    ];
    for (index, (name, open, content, close)) in expected.into_iter().enumerate() {
        let block = &capture.blocks[index];
        let retained = &observed.blocks[index];
        assert_eq!(
            (block.name, block.open_tag, block.content, block.close_tag),
            (name, open, content, Some(close))
        );
        assert_eq!(
            (
                retained.name,
                retained.open_tag,
                retained.content,
                retained.close_tag
            ),
            (name, open, content, Some(close))
        );
        assert_eq!(block.name.as_ptr(), retained.name.as_ptr());
        assert_eq!(&*block.attrs, attrs[index]);
        assert_eq!(&*retained.attrs, attrs[index]);
        for (left, right) in block.attrs.iter().zip(&retained.attrs) {
            assert_eq!(left.name.as_ptr(), right.name.as_ptr());
            assert_eq!(left.value.map(str::as_ptr), right.value.map(str::as_ptr));
        }
    }
    let admitted = owner.admitted().unwrap();
    let names = admitted.template().unwrap().frame_names().unwrap();
    assert_eq!(
        (names.opening(), names.closing()),
        (Span::new(5, 13), Span::new(29, 37))
    );
    assert_eq!(admitted.setup().unwrap().block().source(), "const x=1");
    assert_eq!(admitted.styles().next().unwrap().block().source(), ".x{}");
}

#[test]
fn original_missing_self_closed_and_uncertain_frames_never_admit_names() {
    for (source, uncertain, self_closed) in [
        ("<template><p>", false, false),
        ("<template/>", false, true),
        ("<template>{{ `value ${name}` }}</template>", true, false),
        ("<template>{{ value", true, false),
    ] {
        let arena = Allocator::default();
        let capture = Vue.split(&arena, source);
        let owner = Vue.observe_descriptor(&arena, source, options());
        assert_eq!(&*capture.errors, &*owner.container().errors);
        let block = &capture.blocks[0];
        if self_closed {
            let end = source.len() as u32;
            assert_eq!(block.close_tag, Some(Span::new(end, end)));
            assert!(capture.errors.is_empty());
        } else {
            assert!(block.close_tag.is_none());
            assert!(
                capture
                    .errors
                    .iter()
                    .any(|error| error.code == ContainerErrorCode::MissingCloseTag)
            );
        }
        assert_eq!(
            capture
                .errors
                .iter()
                .any(|error| error.code == ContainerErrorCode::UncertainInterpolation),
            uncertain
        );
        let refused = owner.admitted().unwrap_err();
        assert!(
            refused
                .issues()
                .iter()
                .any(|issue| issue.code == Code::UnsupportedBoundary)
        );
        assert!(owner.template.as_ref().unwrap().names.is_none());
    }
}

#[test]
fn original_opening_spelling_and_post_slash_or_name_suffix_policy_stay_refused() {
    for (source, code) in [
        ("<Template></Template>", Code::UnsupportedBlockSpelling),
        ("<template></ template>", Code::UnsupportedBoundary),
        ("<template></templateX>", Code::UnsupportedBoundary),
    ] {
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, source, options());
        let refused = owner.admitted().unwrap_err();
        assert!(refused.issues().iter().any(|issue| issue.code == code));
    }
}

#[test]
fn original_descriptor_policy_refusals_cannot_promote_retained_frame_metadata() {
    let mut old = options();
    old.version = VueVersion::V2;
    let mut petite = options();
    petite.dialect = VueDialect::PetiteVue;
    let mut experimental = options();
    experimental.template.experimental_in_tag_comments = true;
    for (source, options, code) in [
        ("<template>x</template>", old, Code::UnsupportedVersion),
        ("<template>x</template>", petite, Code::UnsupportedDialect),
        (
            "<template>x</template>",
            experimental,
            Code::UnsupportedOptions,
        ),
        (
            "<template src='part.html'></template>",
            options(),
            Code::ExternalSource,
        ),
        (
            "<template lang=pug>x</template>",
            options(),
            Code::UnsupportedLanguage,
        ),
        (
            "<template>x</template><template>y</template>",
            options(),
            Code::DuplicateRole,
        ),
    ] {
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, source, options);
        let refused = owner.admitted().unwrap_err();
        assert!(refused.issues().iter().any(|issue| issue.code == code));
    }
}

#[test]
fn original_script_and_style_roles_stay_unchanged_without_a_template() {
    let source = "<style scoped>.x{}</style><script lang=ts>export const x=1</script><script setup lang=ts>const y=2</script>";
    let arena = Allocator::default();
    let owner = Vue.observe_descriptor(&arena, source, options());
    let admitted = owner.admitted().unwrap();
    assert!(admitted.template().is_none());
    assert!(owner.template.is_none());
    let ordinary = admitted.ordinary().unwrap();
    let setup = admitted.setup().unwrap();
    assert_eq!(
        (ordinary.container_index(), setup.container_index()),
        (1, 2)
    );
    assert_eq!(
        (ordinary.role(), setup.role()),
        (ScriptRole::Ordinary, ScriptRole::Setup)
    );
    assert_eq!(
        (ordinary.lang(), setup.lang(), admitted.template_lang()),
        (Lang::Ts, Lang::Ts, Lang::Ts)
    );
    assert_eq!(ordinary.block().source(), "export const x=1");
    assert_eq!(setup.block().source(), "const y=2");
    let style = admitted.styles().next().unwrap();
    assert_eq!(
        (style.container_index(), style.block().source()),
        (0, ".x{}")
    );
    assert_eq!(style.source().as_ptr(), source.as_ptr());
    let style_only = Vue.observe_descriptor(&arena, "<style>.x{}</style>", options());
    assert!(
        style_only
            .admitted()
            .unwrap_err()
            .issues()
            .iter()
            .any(|issue| issue.code == Code::MissingComponentBlock)
    );
}
