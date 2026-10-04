//! Whole-SFC refusals keep the original source, policies and actual observed prefix.

use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{
    ExpressionRefusal, NativeSfcBlockRole, NativeSfcObservation, NativeSfcOptions,
    NativeSfcRefusal, NativeTemplateRefusal, ObservedNativeTemplateRefusal, observe_native_sfc_in,
};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, Span};
use vize_l1::SurfaceChild;
use vize_l1::container::ContainerErrorCode;
use vize_l1::container::vue::{DescriptorIssue, DescriptorIssueCode as Code, ScriptRole};
use vize_l1::embed::syntax::EmbedHole;
use vize_l1::markup::NativeInterpolationError;

fn refused(owner: &NativeSfcObservation<'_>, source: &str, expected: NativeSfcRefusal) {
    assert!(core::ptr::eq(owner.source(), source));
    assert!(core::ptr::eq(owner.descriptor().source(), source));
    assert_eq!(owner.options().descriptor, owner.descriptor().options());
    let issues = owner.descriptor().issues().to_vec();
    let errors = owner.descriptor().container().errors.to_vec();
    for _ in 0..2 {
        assert_eq!(owner.refusal(), Some(expected));
        assert_eq!(owner.document().unwrap_err(), expected);
        assert_eq!(owner.format().unwrap_err(), expected);
    }
    assert_eq!(owner.descriptor().issues(), issues);
    assert_eq!(owner.descriptor().container().errors.as_slice(), errors);
    for operand in owner.operands() {
        assert!(core::ptr::eq(
            operand.syntax().source().authored_root(),
            source
        ));
        assert_eq!(operand.content_span().slice(source), operand.raw_content());
    }
}

fn span(source: &str, spelling: &str) -> Span {
    let start = source.rfind(spelling).unwrap() as u32;
    Span::new(start, start + spelling.len() as u32)
}

fn late(refusal: NativeTemplateRefusal) -> NativeSfcRefusal {
    NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document { index: 1, refusal })
}

#[test]
fn even_empty_script_setup_and_style_roles_refuse_before_any_template_observation() {
    let sources = [
        "<script></script><template>{{a+}}</template>",
        "<template>{{a+}}</template><script setup></script>",
        "<style scoped></style><template>{{a+}}</template>",
        "<template>{{a+}}</template><style></style><script></script>",
        "<script setup></script><template>{{a+}}</template><style></style><script></script>",
    ];
    let evidence = [
        (
            0,
            "<script>",
            NativeSfcBlockRole::Script(ScriptRole::Ordinary),
        ),
        (
            1,
            "<script setup>",
            NativeSfcBlockRole::Script(ScriptRole::Setup),
        ),
        (0, "<style scoped>", NativeSfcBlockRole::Style),
        (1, "<style>", NativeSfcBlockRole::Style),
        (
            0,
            "<script setup>",
            NativeSfcBlockRole::Script(ScriptRole::Setup),
        ),
    ];
    for (source, (index, spelling, role)) in sources.into_iter().zip(evidence) {
        let arena = Allocator::default();
        let owner = observe_native_sfc_in(&arena, source, NativeSfcOptions::default());
        assert!(owner.descriptor().admitted().is_ok());
        let expected = NativeSfcRefusal::UnsupportedBlock {
            index,
            span: span(source, spelling),
            role,
        };
        refused(&owner, source, expected);
        assert!(owner.selected().is_none());
        assert!(owner.operands().is_empty());
        assert!(owner.interpolation_failure().is_none());
        assert_eq!(
            owner.descriptor().container().blocks[index]
                .content
                .slice(source),
            ""
        );
    }
}

#[test]
fn original_descriptor_policy_issues_precede_roles_with_exact_authored_evidence() {
    let sources = [
        "<template></template><custom>raw</custom>",
        "<template lang='pug'>raw</template>",
        "<template lang='h&#116;ml'></template>",
        "<script src='raw.js'></script><template></template>",
        "<template lang='html' lang='html'></template>",
        "<template></template><template></template>",
        "<Template></Template>",
    ];
    let evidence = [
        (Code::UnsupportedBlock, 1, "<custom>"),
        (Code::UnsupportedLanguage, 0, "lang='pug'"),
        (Code::EncodedLanguage, 0, "lang='h&#116;ml'"),
        (Code::ExternalSource, 0, "src='raw.js'"),
        (Code::DuplicateAttribute, 0, "lang='html'"),
        (Code::DuplicateRole, 1, "<template>"),
        (Code::UnsupportedBlockSpelling, 0, "<Template>"),
    ];
    for (source, (code, index, spelling)) in sources.into_iter().zip(evidence) {
        let arena = Allocator::default();
        let owner = observe_native_sfc_in(&arena, source, NativeSfcOptions::default());
        refused(&owner, source, NativeSfcRefusal::Descriptor);
        assert_eq!(
            owner.descriptor().issues().first(),
            Some(&DescriptorIssue {
                code,
                container_index: Some(index),
                span: span(source, spelling)
            })
        );
        assert!(owner.descriptor().admitted().is_err());
        assert!(owner.selected().is_none());
        assert!(owner.operands().is_empty());
        assert!(owner.interpolation_failure().is_none());
        if code == Code::DuplicateRole {
            assert_eq!(
                owner.descriptor().container().errors[0].code,
                ContainerErrorCode::DuplicateBlock
            );
            assert_eq!(
                owner.descriptor().container().errors[0].offset,
                span(source, spelling).start
            );
        } else {
            assert!(owner.descriptor().container().errors.is_empty());
        }
    }
}

#[test]
fn native_version_dialect_and_surface_options_are_retained_without_fallback() {
    let source = "<style></style><template>{{1n}}</template>";
    for (version, dialect, experimental, code) in [
        (
            VueVersion::V2,
            VueDialect::Vue,
            false,
            Code::UnsupportedVersion,
        ),
        (
            VueVersion::V3,
            VueDialect::PetiteVue,
            false,
            Code::UnsupportedDialect,
        ),
        (
            VueVersion::V3,
            VueDialect::Vue,
            true,
            Code::UnsupportedOptions,
        ),
    ] {
        let mut options = NativeSfcOptions::default();
        options.descriptor.version = version;
        options.descriptor.dialect = dialect;
        options.descriptor.template.experimental_in_tag_comments = experimental;
        let arena = Allocator::default();
        let owner = observe_native_sfc_in(&arena, source, options);
        refused(&owner, source, NativeSfcRefusal::Descriptor);
        assert_eq!(owner.options(), options);
        assert_eq!(
            owner.descriptor().issues(),
            &[DescriptorIssue {
                code,
                container_index: None,
                span: Span::new(0, 0)
            }]
        );
        assert!(owner.selected().is_none());
        assert!(owner.operands().is_empty());
    }
}

#[test]
fn late_mapped_unsupported_expression_keeps_current_ast_map_and_later_original_bytes() {
    let arena = Allocator::default();
    let source = "<!--前--><template>{{1n}}<p>{{a&#43;f(...b)}}</p>{{later}}</template>tail";
    let owner = observe_native_sfc_in(&arena, source, NativeSfcOptions::default());
    assert_eq!(owner.operands().len(), 2);
    let current = &owner.operands()[1];
    assert_eq!(current.raw_content(), "a&#43;f(...b)");
    assert_eq!(current.syntax().hole(), None);
    let Expression::BinaryExpression(binary) = current.syntax().expression().unwrap() else {
        panic!("actual mapped Binary")
    };
    let Expression::CallExpression(call) = &binary.right else {
        panic!("actual Call")
    };
    let span = current.syntax().decoded_span(call.span).unwrap();
    assert_eq!(span, Span::new(2, 9));
    let root = core::ptr::from_ref(current.syntax().expression().unwrap());
    let view = current.syntax().source();
    let map = view.decode_map().unwrap().segments();
    let segments = map.to_vec();
    refused(
        &owner,
        source,
        late(NativeTemplateRefusal::Expression {
            offset: 9,
            refusal: ExpressionRefusal::UnsupportedNode { span },
        }),
    );
    assert_eq!(
        core::ptr::from_ref(current.syntax().expression().unwrap()),
        root
    );
    assert_eq!(view.text(), "a+f(...b)");
    assert!(core::ptr::eq(
        current.syntax().source().decode_map().unwrap().segments(),
        map
    ));
    assert_eq!(map, segments.as_slice());
    let selected = owner.selected().unwrap();
    let SurfaceChild::Interpolation(later) = selected.children().nth(2).unwrap().surface() else {
        panic!("untouched later child")
    };
    assert_eq!(later.content.text, "later");
    assert_eq!(
        selected.component().block().source(),
        "{{1n}}<p>{{a&#43;f(...b)}}</p>{{later}}"
    );
    assert!(owner.interpolation_failure().is_none());
}

#[test]
fn late_syntax_hole_keeps_actual_comment_diagnostic_storage_and_no_partial_output() {
    let arena = Allocator::default();
    let source = "<template>{{1n}}<p>{{/*x*/ a+}}</p>{{later}}</template>tail";
    let owner = observe_native_sfc_in(&arena, source, NativeSfcOptions::default());
    assert_eq!(owner.operands().len(), 2);
    let current = &owner.operands()[1];
    let syntax = current.syntax();
    let hole = syntax.hole();
    assert_eq!(hole, Some(EmbedHole::Syntax));
    assert!(syntax.expression().is_none());
    let comment = syntax.comments().next().unwrap();
    assert_eq!(syntax.comments().count(), 1);
    assert_eq!(comment.text().unwrap(), "/*x*/");
    let comment_pointer = comment.text().unwrap().as_ptr();
    let authored = comment.authored_span().unwrap();
    let diagnostics = || {
        syntax
            .diagnostics()
            .map(|diagnostic| {
                (
                    diagnostic.message().as_ptr(),
                    diagnostic.message().to_owned(),
                    diagnostic
                        .labels()
                        .map(|label| (label.decoded_span(), label.authored_span()))
                        .collect::<std::vec::Vec<_>>(),
                )
            })
            .collect::<std::vec::Vec<_>>()
    };
    let original_diagnostics = diagnostics();
    assert!(!original_diagnostics.is_empty());
    refused(
        &owner,
        source,
        late(NativeTemplateRefusal::OperandRejected {
            offset: 9,
            index: 1,
            hole,
        }),
    );
    assert_eq!(
        syntax.comments().next().unwrap().text().unwrap().as_ptr(),
        comment_pointer
    );
    assert_eq!(authored.slice(source), "/*x*/");
    assert_eq!(diagnostics(), original_diagnostics);
    assert_eq!(current.raw_content(), "/*x*/ a+");
    assert!(owner.interpolation_failure().is_none());
    assert_eq!(
        owner.selected().unwrap().component().block().source(),
        "{{1n}}<p>{{/*x*/ a+}}</p>{{later}}"
    );
}

#[test]
fn genuine_recovered_parent_failure_keeps_actual_failure_without_a_fabricated_current() {
    let arena = Allocator::default();
    let source = "<template>{{1n}}<p>{{b}}</template>tail";
    let owner = observe_native_sfc_in(&arena, source, NativeSfcOptions::default());
    assert!(owner.descriptor().admitted().is_ok());
    assert!(
        owner
            .selected()
            .unwrap()
            .component()
            .carrier()
            .errors
            .is_empty()
    );
    assert_eq!(owner.operands().len(), 1);
    assert_eq!(owner.operands()[0].raw_content(), "1n");
    let actual = owner.interpolation_failure().unwrap();
    let pointer = core::ptr::from_ref(actual);
    assert_eq!(actual.kind(), NativeInterpolationError::RecoveredComponent);
    assert!(actual.syntax().is_none());
    refused(
        &owner,
        source,
        NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Interpolation {
            offset: 9,
            index: 1,
            kind: NativeInterpolationError::RecoveredComponent,
        }),
    );
    assert_eq!(
        core::ptr::from_ref(owner.interpolation_failure().unwrap()),
        pointer
    );
    assert_eq!(
        owner.selected().unwrap().component().block().source(),
        "{{1n}}<p>{{b}}"
    );
}
