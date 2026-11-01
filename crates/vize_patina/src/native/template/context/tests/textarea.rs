use super::*;
use crate::native::{
    admission,
    template::{NativeTemplateOwnerProfile as Profile, NativeTemplateTextProfile},
};

#[test]
fn textarea_capability_intersection_never_expands_strict_or_select_only_callbacks() {
    for left in [
        Profile::Strict,
        Profile::ClosedSelect,
        Profile::ClosedSelectAndTextarea,
    ] {
        for right in [
            Profile::Strict,
            Profile::ClosedSelect,
            Profile::ClosedSelectAndTextarea,
        ] {
            let expected = if left == Profile::Strict || right == Profile::Strict {
                Profile::Strict
            } else if left == Profile::ClosedSelect || right == Profile::ClosedSelect {
                Profile::ClosedSelect
            } else {
                Profile::ClosedSelectAndTextarea
            };
            assert_eq!(left.intersect(right), expected);
            assert_eq!(right.intersect(left), expected);
        }
    }
}

#[test]
fn original_closed_textarea_proof_is_post_body_foreign_safe_and_callback_bound() {
    let allocator = Allocator::new();
    let source = "<textarea>plain</textarea>";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let root = NativeLintComponent::parse_in(&allocator, block).unwrap();
    let foreign = NativeLintComponent::parse_in(&allocator, block).unwrap();
    let linter = Linter::new();
    let mut owners = 0;
    admission::children_with_owner_profile(
        root.component(),
        root.children(),
        NativeTemplateAttributeProfile::Bindings,
        NativeTemplateTextProfile::LiteralText,
        Profile::ClosedSelectAndTextarea,
        &mut |element| {
            let owner = element.closed_textarea_owner().unwrap();
            assert!(element.closed_select_owner().is_none());
            owners += 1;
            let mut context = NativeTemplateLintContext::new(&linter, &root, "Textarea.vue");
            for profile in [Profile::Strict, Profile::ClosedSelect] {
                context.current_owner_profile = profile;
                assert_eq!(
                    context.textarea_opening_range(&owner),
                    Err(NativeTemplateLintRefusal::UnsupportedContext {
                        span: Span::new(0, 10)
                    })
                );
            }
            context.current_owner_profile = Profile::ClosedSelectAndTextarea;
            assert_eq!(context.textarea_opening_range(&owner), Ok(Span::new(0, 10)));
            let mut context = NativeTemplateLintContext::new(&linter, &foreign, "Foreign.vue");
            context.current_owner_profile = Profile::ClosedSelectAndTextarea;
            assert_eq!(
                context.warn_textarea_with_help(&owner, "message", &[], "help"),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert!(context.finish().diagnostics.is_empty());
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(owners, 1);
}

#[test]
fn invalid_hidden_body_never_dispatches_owner_and_a_bare_header_proves_nothing() {
    let allocator = Allocator::new();
    for source in [
        "<textarea hidden>&amp;</textarea>",
        "<textarea hidden>{{value}}</textarea>",
    ] {
        let root = NativeLintComponent::parse_in(
            &allocator,
            SourceRoot::new(source).unwrap().whole_block(),
        )
        .unwrap();
        let mut callbacks = 0;
        assert!(
            admission::children_with_owner_profile(
                root.component(),
                root.children(),
                NativeTemplateAttributeProfile::Bindings,
                NativeTemplateTextProfile::LiteralText,
                Profile::ClosedSelectAndTextarea,
                &mut |_| {
                    callbacks += 1;
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(callbacks, 0);
    }
    let source = "<textarea hidden></textarea>";
    let root =
        NativeLintComponent::parse_in(&allocator, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let original = root.children().next().unwrap().into_element().unwrap();
    let view = NativeTemplateElement::new(original, vize_l0::SmallVec::new());
    let linter = Linter::new();
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Textarea.vue");
    context.current_owner_profile = Profile::ClosedSelectAndTextarea;
    assert!(view.closed_textarea_owner().is_none());
    use crate::native::template::NativeTemplateRule;
    assert_eq!(
        crate::rules::a11y::FormControlHasLabel.run_on_element(&mut context, &view),
        Err(NativeTemplateLintRefusal::UnsupportedContext {
            span: Span::new(0, 17)
        })
    );
    assert!(context.finish().diagnostics.is_empty());
}
