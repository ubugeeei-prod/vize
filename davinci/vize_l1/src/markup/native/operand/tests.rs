use super::{NativeAttributeOperandError, NativeConditionKind};
use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::embed::{Lang, syntax::EmbedHole};
use crate::markup::{NativeTemplateComponent, NativeTemplateGrammar};
use oxc_span::GetSpan;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};

fn selected<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateComponent<'a>, &'static str> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::SurfaceParseOptions::default(),
        },
    );
    let admitted = descriptor
        .admitted()
        .map_err(|_| "fixture descriptor refused")?;
    NativeTemplateComponent::parse_in(arena, admitted)
        .map_err(|_| "fixture component parse failed")?
        .ok_or("fixture has no selected template")
}

#[test]
fn same_original_header_retains_complete_entity_map_and_stock_root() {
    let arena = Allocator::default();
    let source = "<!--前--><template><p v-if='msg &amp;&amp; ok'>kept</p></template>";
    let owner = selected(&arena, source).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let operand = owner
        .observe_attribute_expression(element.attributes().next().unwrap())
        .unwrap();
    assert_eq!(operand.kind(), NativeConditionKind::If);
    assert_eq!(operand.raw_value(), "msg &amp;&amp; ok");
    assert_eq!(operand.value_span().slice(source), operand.raw_value());
    assert_eq!(operand.name_span().slice(source), "v-if");
    assert_eq!(operand.syntax().source().text(), "msg && ok");
    assert!(operand.syntax().source().decode_map().is_some());
    assert!(core::ptr::eq(
        operand.syntax().source().authored_root(),
        source
    ));
    let original = operand.syntax().expression().unwrap();
    let projected = operand
        .admitted_for(&owner, element.attributes().next().unwrap())
        .unwrap();
    assert!(core::ptr::eq(projected.selected(), &owner));
    assert!(core::ptr::eq(
        projected.attribute().element(),
        element.surface()
    ));
    assert!(core::ptr::eq(
        projected.expression().unwrap().expression(),
        original
    ));
    assert_eq!(
        operand.syntax().authored_span(original.span()).unwrap(),
        operand.value_span()
    );
}

#[test]
fn owners_move_and_pending_storage_grows_before_short_header_admission() {
    let arena = Allocator::default();
    let source = "<template><p v-if='ready'/></template>";
    let owner = selected(&arena, source).unwrap();
    let operand = {
        let element = owner.children().next().unwrap().into_element().unwrap();
        owner
            .observe_attribute_expression(element.attributes().next().unwrap())
            .unwrap()
    };
    let original = operand.syntax().expression().unwrap();
    let mut parked = alloc::vec::Vec::new();
    parked.push(operand);
    parked.reserve(32);
    let moved = core::hint::black_box(owner);
    let element = moved.children().next().unwrap().into_element().unwrap();
    {
        let projected = parked
            .first()
            .unwrap()
            .admitted_for(&moved, element.attributes().next().unwrap())
            .unwrap();
        assert!(core::ptr::eq(
            projected.expression().unwrap().expression(),
            original
        ));
    }
    let moved_again = core::hint::black_box(moved);
    assert_eq!(moved_again.children().len(), 1);
    assert!(
        parked
            .pop()
            .unwrap()
            .syntax()
            .admitted_expression()
            .is_some()
    );
}

#[test]
fn sibling_attribute_and_independent_same_input_parse_do_not_admit() {
    let arena = Allocator::default();
    let source = "<template><p v-if='ready' v-else-if='ready'/></template>";
    let owner = selected(&arena, source).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let operand = owner
        .observe_attribute_expression(element.attributes().next().unwrap())
        .unwrap();
    assert!(
        operand
            .admitted_for(&owner, element.attributes().nth(1).unwrap())
            .is_none()
    );
    let other = selected(&arena, source).unwrap();
    let other_element = other.children().next().unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&other, other_element.attributes().next().unwrap())
            .is_none()
    );
    assert!(
        matches!(owner.observe_attribute_expression(other_element.attributes().next().unwrap()),
        Err(error) if error.kind() == NativeAttributeOperandError::ForeignComponent)
    );
    let copied = vize_l0::String::from(source);
    let foreign = selected(&arena, copied.as_str()).unwrap();
    let foreign_element = foreign.children().next().unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(&foreign, foreign_element.attributes().next().unwrap())
            .is_none()
    );
}

#[test]
fn selected_setup_typescript_profile_is_intrinsic_to_the_operand() {
    let arena = Allocator::default();
    let owner = selected(
        &arena,
        "<template><p v-if='value as boolean'/></template><script setup lang=ts>const value=true</script>",
    )
    .unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let operand = owner
        .observe_attribute_expression(element.attributes().next().unwrap())
        .unwrap();
    assert_eq!(owner.grammar(), NativeTemplateGrammar::TypeScriptModule);
    assert_eq!(operand.syntax().grammar().lang, Lang::Ts);
    assert!(operand.syntax().source_type().is_typescript());
    assert!(operand.syntax().source_type().is_module());
    assert!(
        operand
            .admitted_for(&owner, element.attributes().next().unwrap())
            .is_some()
    );
}

#[test]
fn syntax_holes_keep_all_original_comments_diagnostics_and_source() {
    let arena = Allocator::default();
    let owner = selected(&arena, "<template><p v-if='/*kept*/ ready +'/></template>").unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let operand = owner
        .observe_attribute_expression(element.attributes().next().unwrap())
        .unwrap();
    assert_eq!(operand.syntax().hole(), Some(EmbedHole::Syntax));
    assert_eq!(operand.syntax().comments().count(), 1);
    assert!(operand.syntax().diagnostics().count() > 0);
    assert_eq!(operand.raw_value(), "/*kept*/ ready +");
    assert!(
        operand
            .admitted_for(&owner, element.attributes().next().unwrap())
            .is_none()
    );
}

#[test]
fn empty_present_value_keeps_exact_origin_but_no_expression_admission() {
    let arena = Allocator::default();
    let owner = selected(&arena, "<template><p v-if=''/></template>").unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let operand = owner
        .observe_attribute_expression(element.attributes().next().unwrap())
        .unwrap();
    assert_eq!(operand.raw_value(), "");
    assert_eq!(operand.value_span().start, operand.value_span().end);
    assert!(operand.syntax().hole().is_some());
    assert!(
        operand
            .admitted_for(&owner, element.attributes().next().unwrap())
            .is_none()
    );
}

#[test]
fn unicode_plain_value_remains_the_complete_original_borrow() {
    let arena = Allocator::default();
    let source = "<template><p v-else-if=条件 /></template>";
    let owner = selected(&arena, source).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    let operand = owner
        .observe_attribute_expression(element.attributes().next().unwrap())
        .unwrap();
    assert_eq!(operand.kind(), NativeConditionKind::ElseIf);
    assert_eq!(operand.syntax().grammar().lang, Lang::Js);
    assert!(operand.syntax().source().decode_map().is_none());
    assert!(core::ptr::eq(
        operand.syntax().source().text(),
        operand.raw_value()
    ));
    assert_eq!(operand.value_span().slice(source), "条件");
    assert!(
        operand
            .admitted_for(&owner, element.attributes().next().unwrap())
            .is_some()
    );
}

#[test]
fn recovered_component_is_retained_without_a_conditional_operand() {
    let arena = Allocator::default();
    let source = "<template><p v-if='ready'></template>";
    let owner = selected(&arena, source).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    assert!(owner.component().carrier().errors.is_empty());
    assert!(owner.component().carrier().unsupported.is_empty());
    assert!(
        matches!(owner.observe_attribute_expression(element.attributes().next().unwrap()),
        Err(error) if error.kind() == NativeAttributeOperandError::RecoveredComponent)
    );
    assert!(matches!(
        element.surface().close,
        crate::ElementClose::Missing
    ));
    assert_eq!(owner.component().block().root_source(), source);
}

#[test]
fn verbatim_modifiers_arguments_plain_and_missing_values_are_typed_refusals() {
    let arena = Allocator::default();
    for (source, ordinal, expected) in [
        (
            "<template><p v-pre v-if='ready'/></template>",
            1,
            NativeAttributeOperandError::Verbatim,
        ),
        (
            "<template><p v-if.foo='ready'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p v-if:arg='ready'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p v-if:='ready'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p class='ready'/></template>",
            0,
            NativeAttributeOperandError::UnsupportedDirective,
        ),
        (
            "<template><p v-if/></template>",
            0,
            NativeAttributeOperandError::IncompleteValue,
        ),
    ] {
        let owner = selected(&arena, source).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let failure =
            match owner.observe_attribute_expression(element.attributes().nth(ordinal).unwrap()) {
                Ok(_) => panic!("unexpected admission: {source}"),
                Err(failure) => failure,
            };
        assert_eq!(failure.kind(), expected, "{source}");
        assert!(failure.syntax().is_none());
        assert_eq!(owner.component().block().root_source(), source);
    }
}

#[test]
fn raw_syntax_transfer_keeps_observations_after_selected_owner_drop() {
    let arena = Allocator::default();
    let source = "<template><p v-if='/*kept*/ ready'/></template>";
    let (syntax, original) = {
        let owner = selected(&arena, source).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let operand = owner
            .observe_attribute_expression(element.attributes().next().unwrap())
            .unwrap();
        let original = operand.syntax().expression().unwrap();
        (operand.into_syntax(), original)
    };
    assert!(core::ptr::eq(syntax.expression().unwrap(), original));
    assert_eq!(syntax.comments().count(), 1);
    assert_eq!(syntax.diagnostics().count(), 0);
    assert_eq!(syntax.source().text(), "/*kept*/ ready");
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
}
