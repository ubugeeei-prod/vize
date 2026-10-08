//! Exact whole refusals retain real lower observations and no partial Doc.

use super::*;
use vize_glyph::native_doc::{
    ExpressionRefusal, NativeVue2SfcRefusal as Refusal, Vue2TextDocumentRefusal,
};
use vize_l0::{
    Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::vue::DescriptorIssueCode as Code,
    dialect::vue2::{surface::TextRefusal, text::TextBoundaryKind},
    embed::syntax::EmbedHole,
};

pub(super) fn span(source: &str, raw: &str) -> Span {
    let start = source.find(raw).unwrap() as u32;
    Span::new(start, start + raw.len() as u32)
}
pub(super) fn refused(owner: &NativeVue2SfcObservation<'_>, expected: Refusal) {
    let before = custody::facts(owner);
    assert_eq!(owner.refusal(), Some(expected));
    assert_eq!(owner.document().unwrap_err(), expected);
    assert_eq!(owner.format().unwrap_err(), expected);
    assert!(core::ptr::eq(owner.source(), owner.descriptor().source()));
    assert_eq!(custody::facts(owner), before);
}

#[test]
fn dedicated_native_options_preserve_every_historical_profile_refusal() {
    let source = "<template>{{a|upper}}</template>";
    for version in VueVersion::ALL {
        let arena = Allocator::default();
        let mut opts = NativeVue2SfcOptions::default();
        opts.descriptor.version = version;
        let owner = observe_native_vue2_sfc_in(&arena, source, opts);
        if version == VueVersion::V2 {
            assert!(owner.document().is_ok());
        } else {
            refused(&owner, Refusal::Descriptor);
            assert!(owner.descriptor().component().is_none());
            assert!(
                owner
                    .descriptor()
                    .issues()
                    .iter()
                    .any(|issue| issue.code == Code::UnsupportedVersion)
            );
        }
        assert_eq!(owner.options(), opts);
    }
    for (options, code) in [
        (
            vize_l1::container::vue::DescriptorOptions {
                dialect: VueDialect::PetiteVue,
                ..NativeVue2SfcOptions::default().descriptor
            },
            Code::UnsupportedDialect,
        ),
        (
            vize_l1::container::vue::DescriptorOptions {
                template: SurfaceParseOptions {
                    experimental_in_tag_comments: true,
                },
                ..NativeVue2SfcOptions::default().descriptor
            },
            Code::UnsupportedOptions,
        ),
    ] {
        let arena = Allocator::default();
        let opts = NativeVue2SfcOptions {
            descriptor: options,
            ..NativeVue2SfcOptions::default()
        };
        let owner = observe_native_vue2_sfc_in(&arena, source, opts);
        refused(&owner, Refusal::Descriptor);
        assert_eq!(owner.options(), opts);
        assert!(owner.descriptor().component().is_none());
        assert!(
            owner
                .descriptor()
                .issues()
                .iter()
                .any(|issue| issue.code == code)
        );
    }
}

#[test]
fn complete_script_style_custom_external_duplicate_envelopes_never_expose_a_body_doc() {
    for (source, code) in [
        (
            "<template>{{a}}</template><script></script>",
            Code::UnsupportedScript,
        ),
        (
            "<template>{{a}}</template><style></style>",
            Code::UnsupportedStyle,
        ),
        (
            "<template>{{a}}</template><custom>kept</custom>",
            Code::UnsupportedBlock,
        ),
        (
            "<template src='external.html'></template>",
            Code::ExternalSource,
        ),
        (
            "<template lang='pug'>raw</template>",
            Code::UnsupportedLanguage,
        ),
        (
            "<template>{{a}}</template><template>{{b}}</template>",
            Code::DuplicateRole,
        ),
        (
            "<template lang=html lang=html></template>",
            Code::DuplicateAttribute,
        ),
        ("<Template></Template>", Code::UnsupportedBlockSpelling),
        ("<template/>", Code::UnsupportedBoundary),
    ] {
        let arena = Allocator::default();
        let owner = observe_native_vue2_sfc_in(&arena, source, NativeVue2SfcOptions::default());
        refused(&owner, Refusal::Descriptor);
        assert!(owner.descriptor().component().is_none());
        assert!(
            owner
                .descriptor()
                .issues()
                .iter()
                .any(|issue| issue.code == code),
            "{source}"
        );
    }
}

#[test]
fn conservative_header_refusals_follow_actual_original_visit_order() {
    for tag in [
        "pre",
        "textarea",
        "title",
        "script",
        "style",
        "noscript",
        "template",
        "slot",
        "component",
        "x-widget",
        "b",
    ] {
        let source = format!("<template><div>{{{{a+b}}}}</div><{tag}>kept</{tag}></template>");
        let arena = Allocator::default();
        let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
        refused(
            &owner,
            Refusal::UnsupportedElement {
                // The original nested opening follows the fixed 28-byte envelope/body prefix.
                span: Span::new(28, 29 + tag.len() as u32),
            },
        );
        assert!(owner.descriptor().component().is_some());
    }
    for name in [
        "v-if",
        "v-else",
        "v-else-if",
        "v-on:click",
        "v-bind:id",
        "v-pre.foo",
        ":id",
        "@click",
        "#slot",
        ".id",
        "slot",
        "slot-scope",
        "scope",
        "is",
        "inline-template",
        "key",
        "ref",
        "ref-in-for",
    ] {
        let source = format!(
            "<template><div>{{{{a+b}}}}</div><span {name}='raw'>{{{{b+c}}}}</span></template>"
        );
        let arena = Allocator::default();
        let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
        refused(
            &owner,
            Refusal::UnsupportedAttribute {
                span: span(&source, name),
            },
        );
        assert_eq!(owner.descriptor().component().unwrap().bindings().len(), 2);
    }
    let source = "<template><div v-pre>{{bad+}}</div></template>";
    let arena = Allocator::default();
    let owner = observe_native_vue2_sfc_in(&arena, source, NativeVue2SfcOptions::default());
    refused(
        &owner,
        Refusal::UnsupportedElement {
            span: span(source, "<div"),
        },
    );
    assert!(
        owner
            .descriptor()
            .component()
            .unwrap()
            .bindings()
            .is_empty()
    );
}

#[test]
fn real_text_holes_comment_list_and_encoded_delimiter_refusals_remain_original() {
    for (bad, refusal) in [
        ("{{a+}}", TextRefusal::NativeHole(EmbedHole::Syntax)),
        ("{{a|add(1+)}}", TextRefusal::NativeHole(EmbedHole::Syntax)),
        (
            "{{a/*keep*/+b}}",
            TextRefusal::Boundary(TextBoundaryKind::CommentSyntax),
        ),
        (
            "{{a&#47;&#42;keep&#42;&#47;+b}}",
            TextRefusal::Boundary(TextBoundaryKind::CommentSyntax),
        ),
        (
            "{{a|add(,2)}}",
            TextRefusal::Boundary(TextBoundaryKind::UnsupportedArgumentList),
        ),
        (
            "{{&#13;a}}",
            TextRefusal::Boundary(TextBoundaryKind::HistoricalLineSeparator),
        ),
    ] {
        let source = format!("<template><div>{{{{b+c}}}}{bad}{{{{a+b}}}}</div></template>");
        let arena = Allocator::default();
        let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
        refused(
            &owner,
            Refusal::Text {
                span: span(&source, bad),
                refusal,
            },
        );
        let component = owner.descriptor().component().unwrap();
        assert_eq!(component.bindings().len(), 3);
        let child = component
            .children()
            .next()
            .unwrap()
            .children()
            .unwrap()
            .nth(1)
            .unwrap();
        assert_eq!(component.text_for(child).unwrap_err(), refusal);
    }
    let source = "<template>&#123;&#123;raw&#125;&#125;</template>";
    let arena = Allocator::default();
    let owner = observe_native_vue2_sfc_in(&arena, source, NativeVue2SfcOptions::default());
    refused(
        &owner,
        Refusal::ComponentBoundary {
            span: span(source, "&#123;"),
            kind: TextBoundaryKind::EncodedDelimiter,
        },
    );
    assert!(
        owner
            .descriptor()
            .component()
            .unwrap()
            .bindings()
            .is_empty()
    );
}

#[test]
fn unsupported_original_operands_and_doc_depth_refuse_the_complete_whole_owner() {
    for raw in ["/x/", "`a`", "a?.b"] {
        let source = format!("<template><div>{{{{b+c}}}}{{{{{raw}}}}}{{{{a+b}}}}</div></template>");
        let arena = Allocator::default();
        let owner = observe_native_vue2_sfc_in(&arena, &source, NativeVue2SfcOptions::default());
        refused(
            &owner,
            Refusal::TextDoc(Vue2TextDocumentRefusal::Operand {
                span: span(&source, raw),
                error: ExpressionRefusal::UnsupportedNode {
                    span: Span::new(0, raw.len() as u32),
                },
            }),
        );
        assert!(
            owner.descriptor().component().unwrap().bindings()[1]
                .admitted()
                .is_some()
        );
    }
    for depth in [16, 17] {
        let raw = format!("{}a", "!".repeat(depth));
        let source = format!("<template><div>{{{{{raw}}}}}</div></template>");
        let arena = Allocator::default();
        let owner = observe_native_vue2_sfc_in(&arena, &source, options(200, 2, LineEnding::Lf));
        if depth == 16 {
            fixed(
                &source,
                &format!(
                    "<template><div>{{{{{}a}}}}</div></template>",
                    "! ".repeat(16)
                ),
                options(200, 2, LineEnding::Lf),
            );
        } else {
            refused(
                &owner,
                Refusal::TextDoc(Vue2TextDocumentRefusal::Operand {
                    span: span(&source, &raw),
                    error: ExpressionRefusal::DepthLimit {
                        span: Span::new(17, 18),
                    },
                }),
            );
        }
    }
}
