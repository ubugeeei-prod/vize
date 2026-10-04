use super::{assert_output, options, policy::empty_refusal};
use vize_glyph::native_doc::{
    LineEnding, NativeSfcBlockRole, NativeSfcDirectivePolicy, NativeSfcRefusal,
    NativeTemplateRefusal, NativeTemplateValuePolicy, ObservedNativeTemplateRefusal,
    native_template_document, observe_native_sfc_in, observed_native_template_document,
    observed_native_template_document_with_policy, print,
};
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, Span};
use vize_l1::container::vue::{DescriptorIssue, DescriptorIssueCode, ScriptRole};
use vize_l1::markup::NativeInterpolationError;

#[test]
fn v_pre_and_bare_template_opaque_routes_keep_original_values_without_minting_bindings() {
    let arena = Allocator::default();
    let policy = options(200, 2, LineEnding::Lf);
    let source = "<template><p v-pre :id='a+b'>{{1n}}</p></template>";
    let owner = observe_native_sfc_in(&arena, source, policy);
    empty_refusal(
        &owner,
        source,
        policy,
        NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
            index: 0,
            refusal: NativeTemplateRefusal::DirectiveValue {
                span: Span::new(24, 27),
            },
        }),
    );
    let selected = owner.selected().unwrap();
    let opaque = observed_native_template_document_with_policy(
        selected,
        &arena,
        NativeTemplateValuePolicy::PreserveOpaque,
    )
    .unwrap();
    assert_eq!(
        print(opaque.document(), &policy.print),
        "<p v-pre :id='a+b'>{{1n}}</p>"
    );
    assert!(opaque.binding_operands().is_empty());
    assert!(opaque.attribute_operands().is_empty());
    assert!(opaque.operands().is_empty());
    let source = "<template><p v-pre>{{a+b}}</p></template>";
    let owner = assert_output(&arena, source, source, policy);
    assert!(owner.binding_operands().is_empty());
    let source = "<template><p :id='a+b'>{{1n}}</p></template>";
    let owner = observe_native_sfc_in(&arena, source, policy);
    let selected = owner.selected().unwrap();
    let opaque = observed_native_template_document(selected, &arena).unwrap();
    assert!(opaque.binding_operands().is_empty());
    assert_eq!(opaque.operands().len(), 1);
    assert_eq!(
        print(opaque.document(), &policy.print),
        "<p :id='a+b'>{{ 1n }}</p>"
    );
    let operands = [&opaque.operands()[0]];
    let bare = native_template_document(selected, &operands, &arena).unwrap();
    assert_eq!(
        print(bare.document(), &policy.print),
        "<p :id='a+b'>{{ 1n }}</p>"
    );
}

#[test]
fn recovered_original_headers_preserve_valued_and_interpolation_refusal_order_across_all_policies()
{
    for directives in [
        NativeSfcDirectivePolicy::Refuse,
        NativeSfcDirectivePolicy::FormatConditionals,
        NativeSfcDirectivePolicy::FormatConditionalsAndStaticBindings,
    ] {
        let arena = Allocator::default();
        let mut policy = options(200, 2, LineEnding::Lf);
        policy.directives = directives;
        let source = "<template><p :id='x'></template>";
        let owner = observe_native_sfc_in(&arena, source, policy);
        empty_refusal(
            &owner,
            source,
            policy,
            NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Document {
                index: 0,
                refusal: NativeTemplateRefusal::DirectiveValue {
                    span: Span::new(18, 19),
                },
            }),
        );
        let source = "<template><p id='x'>{{1n}}</template>";
        let owner = observe_native_sfc_in(&arena, source, policy);
        let refusal = NativeSfcRefusal::Template(ObservedNativeTemplateRefusal::Interpolation {
            offset: 10,
            index: 0,
            kind: NativeInterpolationError::RecoveredComponent,
        });
        assert_eq!(owner.source(), source);
        assert_eq!(owner.options(), policy);
        assert_eq!(owner.refusal(), Some(refusal));
        assert_eq!(owner.document().unwrap_err(), refusal);
        assert_eq!(owner.format().unwrap_err(), refusal);
        assert!(owner.binding_operands().is_empty());
        assert!(owner.attribute_operands().is_empty());
        assert!(owner.operands().is_empty());
        assert!(owner.binding_failure().is_none());
        assert!(owner.attribute_failure().is_none());
        let failure = owner.interpolation_failure().unwrap();
        assert_eq!(failure.kind(), NativeInterpolationError::RecoveredComponent);
        assert!(failure.syntax().is_none());
    }
}

#[test]
fn script_style_and_unknown_outer_roles_refuse_before_any_binding_value_observation() {
    for (source, span, role) in [
        (
            "<script></script><template><p :id='a+b'/></template>",
            Span::new(0, 8),
            NativeSfcBlockRole::Script(ScriptRole::Ordinary),
        ),
        (
            "<style scoped></style><template><p :id='a+b'/></template>",
            Span::new(0, 14),
            NativeSfcBlockRole::Style,
        ),
    ] {
        let arena = Allocator::default();
        let policy = options(200, 2, LineEnding::Lf);
        let owner = observe_native_sfc_in(&arena, source, policy);
        empty_refusal(
            &owner,
            source,
            policy,
            NativeSfcRefusal::UnsupportedBlock {
                index: 0,
                span,
                role,
            },
        );
        assert!(owner.selected().is_none());
        assert_eq!(owner.descriptor().issues(), []);
        assert_eq!(owner.descriptor().container().errors.as_slice(), []);
    }
    let source = "<template><p :id='a+b'/></template><custom>raw</custom>";
    let arena = Allocator::default();
    let policy = options(200, 2, LineEnding::Lf);
    let owner = observe_native_sfc_in(&arena, source, policy);
    empty_refusal(&owner, source, policy, NativeSfcRefusal::Descriptor);
    assert!(owner.selected().is_none());
    assert_eq!(
        owner.descriptor().issues(),
        [DescriptorIssue {
            code: DescriptorIssueCode::UnsupportedBlock,
            container_index: Some(1),
            span: Span::new(35, 43)
        }]
    );
    assert_eq!(owner.descriptor().container().errors.as_slice(), []);
}

#[test]
fn unsupported_native_descriptor_profiles_keep_exact_options_and_first_issue_before_new_policy() {
    let source = "<template><p :id='a+b'/></template>";
    for (version, dialect, experimental, code) in [
        (
            VueVersion::V2,
            VueDialect::Vue,
            false,
            DescriptorIssueCode::UnsupportedVersion,
        ),
        (
            VueVersion::V3,
            VueDialect::PetiteVue,
            false,
            DescriptorIssueCode::UnsupportedDialect,
        ),
        (
            VueVersion::V3,
            VueDialect::Vue,
            true,
            DescriptorIssueCode::UnsupportedOptions,
        ),
    ] {
        let arena = Allocator::default();
        let mut policy = options(200, 2, LineEnding::Lf);
        policy.descriptor.version = version;
        policy.descriptor.dialect = dialect;
        policy.descriptor.template.experimental_in_tag_comments = experimental;
        let owner = observe_native_sfc_in(&arena, source, policy);
        empty_refusal(&owner, source, policy, NativeSfcRefusal::Descriptor);
        assert!(owner.selected().is_none());
        assert_eq!(
            owner.descriptor().issues(),
            [DescriptorIssue {
                code,
                container_index: None,
                span: Span::new(0, 0)
            }]
        );
    }
}
