use super::selected;
use crate::markup::{ArgSyntax, NativeAttributeOperandError, NativeConditionKind};
use vize_l0::{Allocator, Span};

#[test]
fn recognized_unsupported_binding_shapes_refuse_without_preparing_the_original_value() {
    for (source, raw, name) in [
        (
            "<template><p :[key]='broken &amp;'/></template>",
            ":[key]",
            Span::new(13, 19),
        ),
        (
            "<template><p v-bind:[key]='broken &amp;'/></template>",
            "v-bind:[key]",
            Span::new(13, 25),
        ),
        (
            "<template><p :[key]tail='broken &amp;'/></template>",
            ":[key]tail",
            Span::new(13, 23),
        ),
        (
            "<template><p v-bind:[key]tail='broken &amp;'/></template>",
            "v-bind:[key]tail",
            Span::new(13, 29),
        ),
        (
            "<template><p :id.prop='broken &amp;'/></template>",
            ":id.prop",
            Span::new(13, 21),
        ),
        (
            "<template><p :id..camel='broken &amp;'/></template>",
            ":id..camel",
            Span::new(13, 23),
        ),
        (
            "<template><p v-bind:id.camel='broken &amp;'/></template>",
            "v-bind:id.camel",
            Span::new(13, 28),
        ),
        (
            "<template><p .id='broken &amp;'/></template>",
            ".id",
            Span::new(13, 16),
        ),
        (
            "<template><p v-bind='broken &amp;'/></template>",
            "v-bind",
            Span::new(13, 19),
        ),
        (
            "<template><p v-bind:='broken &amp;'/></template>",
            "v-bind:",
            Span::new(13, 20),
        ),
        (
            "<template><p :='broken &amp;'/></template>",
            ":",
            Span::new(13, 14),
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        assert_eq!(attribute.surface().name.text, raw);
        assert_eq!(
            attribute.surface().value.as_ref().unwrap().content.text,
            "broken &amp;"
        );
        let before = arena.allocated_bytes();
        let head = owner.observe_attribute_head(attribute.reborrow()).unwrap();
        assert_eq!(head.name_block().span(), name);
        assert_eq!(head.name_block().source(), raw);
        assert_eq!(head.condition_kind(), None);
        assert_eq!(
            head.static_binding().err(),
            Some(NativeAttributeOperandError::UnsupportedDirective)
        );
        if raw == ":[key]tail" {
            assert_eq!(
                head.directive().unwrap().arg,
                Some(ArgSyntax::Static(Span::new(19, 23)))
            );
        } else if raw == "v-bind:[key]tail" {
            assert_eq!(
                head.directive().unwrap().arg,
                Some(ArgSyntax::Static(Span::new(25, 29)))
            );
        }
        let conditional = head.observe_expression().err().unwrap();
        assert_eq!(
            conditional.kind(),
            NativeAttributeOperandError::UnsupportedDirective
        );
        assert!(conditional.syntax().is_none());
        assert_eq!(arena.allocated_bytes(), before);
        assert_eq!(owner.component().block().root_source(), source);
    }
}

#[test]
fn plain_other_directive_families_and_original_condition_kind_are_not_binding_receipts() {
    for (source, raw, condition) in [
        ("<template><p title='ready'/></template>", "title", None),
        (
            "<template><p v-if='ready'/></template>",
            "v-if",
            Some(NativeConditionKind::If),
        ),
        (
            "<template><p v-else-if='ready'/></template>",
            "v-else-if",
            Some(NativeConditionKind::ElseIf),
        ),
        ("<template><p v-show='ready'/></template>", "v-show", None),
        ("<template><p v-on:id='ready'/></template>", "v-on:id", None),
        ("<template><p @click='ready'/></template>", "@click", None),
        (
            "<template><p #default='ready'/></template>",
            "#default",
            None,
        ),
        (
            "<template><p v-custom:id='ready'/></template>",
            "v-custom:id",
            None,
        ),
        (
            "<template><p v-bindings:id='ready'/></template>",
            "v-bindings:id",
            None,
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        let before = arena.allocated_bytes();
        let head = owner.observe_attribute_head(attribute.reborrow()).unwrap();
        assert_eq!(head.name_block().source(), raw);
        assert_eq!(head.condition_kind(), condition);
        assert!(head.static_binding().unwrap().is_none());
        assert_eq!(arena.allocated_bytes(), before);
        assert!(core::ptr::eq(
            head.attribute().surface(),
            attribute.surface()
        ));
        assert_eq!(owner.component().block().root_source(), source);
    }
}

#[test]
fn original_recovery_and_verbatim_precede_static_shape_value_and_foreign_header_selection() {
    for (source, ordinal, expected) in [
        (
            "<template><p :id='ready'></template>",
            0,
            NativeAttributeOperandError::RecoveredComponent,
        ),
        (
            "<template><p v-pre :id.prop='ready'/></template>",
            1,
            NativeAttributeOperandError::Verbatim,
        ),
        (
            "<template><p v-pre :id/></template>",
            1,
            NativeAttributeOperandError::Verbatim,
        ),
        (
            "<template><p v-pre :id='ready'></template>",
            1,
            NativeAttributeOperandError::RecoveredComponent,
        ),
        (
            "<template><p :id= ></p></template>",
            0,
            NativeAttributeOperandError::RecoveredComponent,
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().nth(ordinal).unwrap();
        let before = arena.allocated_bytes();
        assert_eq!(
            owner.observe_attribute_head(attribute.reborrow()).err(),
            Some(expected)
        );
        let conditional = owner.observe_attribute_expression(attribute).err().unwrap();
        assert_eq!(conditional.kind(), expected);
        assert!(conditional.syntax().is_none());
        assert_eq!(arena.allocated_bytes(), before);
        assert_eq!(owner.component().block().root_source(), source);
    }
    let arena = Allocator::default();
    let recovered_source = "<template><p :id='ready'></template>";
    let recovered = selected(&arena, recovered_source);
    let foreign_source = "<template><p :id='ready'/></template>";
    let foreign = selected(&arena, foreign_source);
    let element = foreign.children().next().unwrap().into_element().unwrap();
    let before = arena.allocated_bytes();
    assert_eq!(
        recovered
            .observe_attribute_head(element.attributes().next().unwrap())
            .err(),
        Some(NativeAttributeOperandError::ForeignComponent)
    );
    assert_eq!(arena.allocated_bytes(), before);
    assert_eq!(
        recovered.component().block().root_source(),
        recovered_source
    );
    assert_eq!(foreign.component().block().root_source(), foreign_source);

    // The original hook's initial dynamic run plus 32 alternating pairs
    // exceeds its 64 distinct-run capacity before any binding selection.
    let name = alloc::format!(":[{}key{}]", "([".repeat(32), "])".repeat(32));
    let source = alloc::format!("<template><p {name}='ready'/></template>");
    let limited = selected(&arena, &source);
    let element = limited.children().next().unwrap().into_element().unwrap();
    let attribute = element.attributes().next().unwrap();
    assert_eq!(attribute.surface().name.text, name.as_str());
    let unsupported = &limited.component().carrier().unsupported;
    assert_eq!(unsupported.len(), 1);
    assert_eq!(
        unsupported[0].error,
        crate::markup::DirectiveNameError::NestingLimit
    );
    assert_eq!(unsupported[0].span, Span::new(3, 3 + name.len() as u32));
    assert_eq!(
        unsupported[0]
            .span
            .slice(limited.component().block().source()),
        name.as_str()
    );
    let before = arena.allocated_bytes();
    assert_eq!(
        limited.observe_attribute_head(attribute).err(),
        Some(NativeAttributeOperandError::RecoveredComponent)
    );
    assert_eq!(arena.allocated_bytes(), before);
    assert_eq!(limited.component().block().root_source(), source.as_str());
}

#[test]
fn supported_static_head_without_value_retains_actual_incomplete_failure_before_preparation() {
    for (source, name, argument) in [
        (
            "<template><p :id/></template>",
            Span::new(13, 16),
            Span::new(14, 16),
        ),
        (
            "<template><p v-bind:id/></template>",
            Span::new(13, 22),
            Span::new(20, 22),
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        assert!(attribute.surface().eq.is_none());
        assert!(attribute.surface().value.is_none());
        let before = arena.allocated_bytes();
        let head = owner.observe_attribute_head(attribute.reborrow()).unwrap();
        assert_eq!(head.name_block().span(), name);
        let binding = head.static_binding().unwrap().unwrap();
        assert_eq!(binding.argument_span(), argument);
        let failure = binding.observe_expression().err().unwrap();
        assert_eq!(failure.kind(), NativeAttributeOperandError::IncompleteValue);
        assert!(failure.syntax().is_none());
        assert_eq!(arena.allocated_bytes(), before);
        let mut parked = alloc::vec![failure];
        parked.reserve(32);
        assert_eq!(
            parked[0].kind(),
            NativeAttributeOperandError::IncompleteValue
        );
        assert!(parked[0].syntax().is_none());
        assert_eq!(owner.component().block().root_source(), source);
    }
}
