use super::{Allocator, selected};
use crate::embed::syntax::EmbedHole;
use crate::markup::NativeInterpolationError;

#[test]
fn sibling_independent_equal_source_and_copied_source_cannot_join_the_origin() {
    let arena = Allocator::default();
    let source = "<template>{{ value }}{{ value }}</template>";
    let owner = selected(&arena, source).unwrap();
    let operand = owner
        .observe_interpolation_expression(owner.children().next().unwrap())
        .unwrap();
    assert!(
        operand
            .admitted_for(&owner, owner.children().nth(1).unwrap())
            .is_none()
    );
    let other = selected(&arena, source).unwrap();
    assert!(
        operand
            .admitted_for(&other, other.children().next().unwrap())
            .is_none()
    );
    assert!(matches!(
        owner.observe_interpolation_expression(other.children().next().unwrap()),
        Err(error) if error.kind() == NativeInterpolationError::ForeignComponent
    ));
    let copied = vize_l0::String::from(source);
    let foreign = selected(&arena, copied.as_str()).unwrap();
    assert!(
        operand
            .admitted_for(&foreign, foreign.children().next().unwrap())
            .is_none()
    );
}

#[test]
fn syntax_holes_retain_complete_content_comments_and_diagnostics() {
    let arena = Allocator::default();
    let source = "<template>{{ \t/*kept*/ value + \n}}</template>";
    let owner = selected(&arena, source).unwrap();
    let operand = owner
        .observe_interpolation_expression(owner.children().next().unwrap())
        .unwrap();
    assert_eq!(operand.syntax().hole(), Some(EmbedHole::Syntax));
    assert_eq!(operand.raw_content(), " \t/*kept*/ value + \n");
    assert_eq!(operand.syntax().source().text(), "/*kept*/ value +");
    assert_eq!(operand.syntax().comments().count(), 1);
    assert!(operand.syntax().diagnostics().count() > 0);
    assert!(
        operand
            .admitted_for(&owner, owner.children().next().unwrap())
            .is_none()
    );
}

#[test]
fn empty_present_interpolations_keep_exact_full_constructs_and_typed_holes() {
    let arena = Allocator::default();
    for (source, raw) in [
        ("<template>{{}}</template>", ""),
        ("<template>{{ \t\n }}</template>", " \t\n "),
    ] {
        let owner = selected(&arena, source).unwrap();
        let operand = owner
            .observe_interpolation_expression(owner.children().next().unwrap())
            .unwrap();
        assert_eq!(operand.raw_content(), raw);
        assert_eq!(operand.content_span().slice(source), raw);
        assert_eq!(operand.syntax().source().text(), "");
        assert!(operand.syntax().hole().is_some());
        assert!(
            operand
                .admitted_for(&owner, owner.children().next().unwrap())
                .is_none()
        );
    }
}

#[test]
fn missing_delimiter_has_no_selected_admission_and_verbatim_text_cannot_parse() {
    let arena = Allocator::default();
    let source = "<template>{{ value</template>";
    let descriptor = super::Vue.observe_descriptor(
        &arena,
        source,
        super::DescriptorOptions {
            version: super::VueVersion::V3,
            dialect: super::VueDialect::Vue,
            template: crate::SurfaceParseOptions::default(),
        },
    );
    assert!(descriptor.admitted().is_err());
    assert!(
        descriptor
            .issues()
            .iter()
            .any(|issue| issue.code
                == crate::container::vue::DescriptorIssueCode::UnsupportedBoundary)
    );
    assert!(core::ptr::eq(descriptor.source(), source));
    let owner = selected(&arena, "<template><p v-pre>{{ value }}</p></template>").unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    assert!(matches!(
        owner.observe_interpolation_expression(parent.children().next().unwrap()),
        Err(error) if error.kind() == NativeInterpolationError::NotInterpolation
    ));
    assert!(parent.surface().open.is_verbatim());
}

#[test]
fn missing_parent_frame_is_refused_even_without_diagnostic_rows() {
    let arena = Allocator::default();
    let source = "<template><p>{{ ready }}</template>";
    let owner = selected(&arena, source).unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    assert!(owner.component().carrier().errors.is_empty());
    assert!(owner.component().carrier().unsupported.is_empty());
    assert!(matches!(
        parent.surface().close,
        crate::ElementClose::Missing
    ));
    assert!(matches!(
        owner.observe_interpolation_expression(parent.children().next().unwrap()),
        Err(error) if error.kind() == NativeInterpolationError::RecoveredComponent
    ));
    assert_eq!(owner.component().block().root_source(), source);
}
