use super::*;
use crate::native::{
    admission,
    template::{NativeTemplateOwnerProfile, NativeTemplateTextProfile},
};

#[test]
fn closed_select_proof_is_post_children_callback_bound_and_original_owner_only() {
    let allocator = Allocator::new();
    let source = "<select><option>雪</option></select>";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let root = NativeLintComponent::parse_in(&allocator, block).unwrap();
    let foreign = NativeLintComponent::parse_in(&allocator, block).unwrap();
    let linter = Linter::new();
    let mut order = Vec::new();
    admission::component(root.component()).unwrap();
    admission::children_with_owner_profile(
        root.component(),
        root.children(),
        NativeTemplateAttributeProfile::Bindings,
        NativeTemplateTextProfile::LiteralText,
        NativeTemplateOwnerProfile::ClosedSelect,
        &mut |element| {
            let tag = element.original().surface().tag();
            order.push(tag);
            if tag == "option" {
                assert!(element.closed_select_owner().is_none());
                return Ok(());
            }
            let owner = element.closed_select_owner().unwrap();
            let mut context = NativeTemplateLintContext::new(&linter, &root, "Select.vue");
            context.current_rule = "a11y/form-control-has-label";
            assert_eq!(
                context.select_opening_range(&owner),
                Err(NativeTemplateLintRefusal::UnsupportedContext {
                    span: Span::new(0, 8)
                })
            );
            context.current_owner_profile = NativeTemplateOwnerProfile::ClosedSelect;
            assert_eq!(context.select_opening_range(&owner), Ok(Span::new(0, 8)));
            assert_eq!(
                context.warn_void_element_with_help(element, "message", &[], "help"),
                Err(NativeTemplateLintRefusal::UnsupportedContext {
                    span: Span::new(0, 8)
                })
            );
            assert!(context.finish().diagnostics.is_empty());
            let mut context = NativeTemplateLintContext::new(&linter, &foreign, "Foreign.vue");
            context.current_owner_profile = NativeTemplateOwnerProfile::ClosedSelect;
            assert_eq!(
                context.warn_select_with_help(&owner, "message", &[], "help"),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert!(context.finish().diagnostics.is_empty());
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(order, ["option", "select"]);
}

#[test]
fn genuine_but_unadmitted_hidden_header_cannot_substitute_for_closed_owner_proof() {
    let allocator = Allocator::new();
    let source = "<select hidden></select>";
    let root =
        NativeLintComponent::parse_in(&allocator, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let original = root.children().next().unwrap().into_element().unwrap();
    let view = NativeTemplateElement::new(original, vize_l0::SmallVec::new());
    let linter = Linter::new();
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Select.vue");
    context.current_owner_profile = NativeTemplateOwnerProfile::ClosedSelect;
    assert!(view.closed_select_owner().is_none());
    use crate::native::template::NativeTemplateRule;
    assert_eq!(
        crate::rules::a11y::FormControlHasLabel.run_on_element(&mut context, &view),
        Err(NativeTemplateLintRefusal::UnsupportedContext {
            span: Span::new(0, 15)
        })
    );
    assert!(context.finish().diagnostics.is_empty());
}

#[test]
fn later_invalid_child_prevents_even_the_hidden_select_callback_and_proof() {
    let allocator = Allocator::new();
    let source = "<select hidden><option>A</option><!--late--></select>";
    let root =
        NativeLintComponent::parse_in(&allocator, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let mut select_callbacks = 0;
    assert!(
        admission::children_with_owner_profile(
            root.component(),
            root.children(),
            NativeTemplateAttributeProfile::Bindings,
            NativeTemplateTextProfile::LiteralText,
            NativeTemplateOwnerProfile::ClosedSelect,
            &mut |element| {
                if element.original().surface().tag() == "select" {
                    select_callbacks += 1;
                }
                Ok(())
            }
        )
        .is_err()
    );
    assert_eq!(select_callbacks, 0);
}
