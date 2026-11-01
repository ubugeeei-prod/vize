use super::*;
use crate::native::{
    admission,
    template::{
        NativeFirstOptionData, NativeTemplateFirstOption, NativeTemplateOptionValue as Value,
        NativeTemplateOwnerProfile as Profile, NativeTemplateTextProfile,
    },
};

fn span(source: &str, authored: &str) -> Span {
    let start = u32::try_from(source.find(authored).unwrap()).unwrap();
    Span::new(start, start + u32::try_from(authored.len()).unwrap())
}

#[test]
fn the_original_first_header_distinguishes_absent_boolean_empty_and_opaque_binding_values() {
    let allocator = Allocator::new();
    for (source, value) in [
        (
            "<select><option>Text</option><option value='a'>A</option></select>",
            Value::Absent,
        ),
        (
            "<select><option value>Text</option></select>",
            Value::Static {
                value: None,
                range: Span::new(16, 21),
            },
        ),
        (
            "<select><option value=''>Text</option></select>",
            Value::Static {
                value: Some(""),
                range: Span::new(16, 24),
            },
        ),
        (
            "<select><option :value='first'>Text</option></select>",
            Value::Bound {
                range: Span::new(16, 30),
            },
        ),
    ] {
        let root = NativeLintComponent::parse_in(
            &allocator,
            SourceRoot::new(source).unwrap().whole_block(),
        )
        .unwrap();
        let linter = Linter::new();
        let mut seen = 0;
        admission::children_with_owner_profile(
            root.component(),
            root.children(),
            NativeTemplateAttributeProfile::Bindings,
            NativeTemplateTextProfile::LiteralText,
            Profile::ClosedSelect,
            &mut |element| {
                if let Some(owner) = element.closed_select_owner() {
                    let mut context = NativeTemplateLintContext::new(&linter, &root, "First.vue");
                    context.current_owner_profile = Profile::ClosedSelect;
                    context.current_callback = Some(0);
                    let first = context.first_option(&owner)?.unwrap();
                    assert_eq!(context.first_option_value(&owner, &first), Ok(value));
                    let end = source[8..].find('>').unwrap() + 9;
                    assert_eq!(
                        context.first_option_opening_range(&owner, &first),
                        Ok(Span::new(8, u32::try_from(end).unwrap()))
                    );
                    seen += 1;
                }
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(seen, 1);
    }
}

#[test]
fn token_issuer_is_the_actual_linter_and_frozen_callback_not_its_metadata_name() {
    let allocator = Allocator::new();
    let source = "<select required><option value='a'>A</option></select>";
    let root =
        NativeLintComponent::parse_in(&allocator, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let linter = Linter::new();
    let other_linter = Linter::new();
    admission::children_with_owner_profile(
        root.component(),
        root.children(),
        NativeTemplateAttributeProfile::StaticOnly,
        NativeTemplateTextProfile::LiteralText,
        Profile::ClosedSelect,
        &mut |element| {
            let Some(owner) = element.closed_select_owner() else {
                return Ok(());
            };
            let mut context = NativeTemplateLintContext::new(&linter, &root, "First.vue");
            assert_eq!(
                context.first_option(&owner).err(),
                Some(NativeTemplateLintRefusal::UnsupportedContext {
                    span: Span::new(0, 17)
                })
            );
            context.current_owner_profile = Profile::ClosedSelect;
            context.current_rule = "same-metadata-name";
            assert_eq!(
                context.first_option(&owner).err(),
                Some(NativeTemplateLintRefusal::UnsupportedContext {
                    span: Span::new(0, 17)
                })
            );
            context.current_callback = Some(0);
            let first = context.first_option(&owner)?.unwrap();
            let expected = Value::Static {
                value: Some("a"),
                range: span(source, "value='a'"),
            };
            assert_eq!(context.first_option_value(&owner, &first), Ok(expected));
            context.current_callback = Some(1);
            assert_eq!(
                context.first_option_value(&owner, &first),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert_eq!(
                context.warn_first_option_with_help(&owner, &first, "message", "help"),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            let own = context.first_option(&owner)?.unwrap();
            assert_eq!(context.first_option_value(&owner, &own), Ok(expected));
            let mut other = NativeTemplateLintContext::new(&other_linter, &root, "Other.vue");
            other.current_owner_profile = Profile::ClosedSelect;
            other.current_callback = Some(1);
            other.current_rule = "same-metadata-name";
            assert_eq!(
                other.first_option_value(&owner, &own),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert!(other.finish().diagnostics.is_empty());
            assert!(context.finish().diagnostics.is_empty());
            Ok(())
        },
    )
    .unwrap();
}

#[test]
fn another_select_in_the_same_component_foreign_component_and_equal_copied_data_cannot_substitute()
{
    let allocator = Allocator::new();
    let source = "<select><option value='a'>A</option></select><select><option value='a'>A</option></select>";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let root = NativeLintComponent::parse_in(&allocator, block).unwrap();
    let foreign = NativeLintComponent::parse_in(&allocator, block).unwrap();
    let linter = Linter::new();
    let mut preceding: Option<NativeFirstOptionData<'_, '_>> = None;
    let mut seen = 0;
    admission::children_with_owner_profile(
        root.component(),
        root.children(),
        NativeTemplateAttributeProfile::StaticOnly,
        NativeTemplateTextProfile::LiteralText,
        Profile::ClosedSelect,
        &mut |element| {
            let Some(owner) = element.closed_select_owner() else {
                return Ok(());
            };
            let mut context = NativeTemplateLintContext::new(&linter, &root, "First.vue");
            context.current_owner_profile = Profile::ClosedSelect;
            context.current_callback = Some(0);
            let first = context.first_option(&owner)?.unwrap();
            // Internal adversarial copies are never public constructors. Even
            // byte-equal fields do not substitute for this stored record.
            let copy = *first.data;
            let copied = NativeTemplateFirstOption {
                data: &copy,
                issuer: first.issuer,
            };
            assert_eq!(
                context.first_option_value(&owner, &copied),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            if let Some(data) = &preceding {
                assert!(core::ptr::eq(data.component, first.data.component));
                assert!(!core::ptr::eq(data.select, first.data.select));
                let other_select = NativeTemplateFirstOption {
                    data,
                    issuer: first.issuer,
                };
                assert_eq!(
                    context.first_option_value(&owner, &other_select),
                    Err(NativeTemplateLintRefusal::SourceMismatch)
                );
            }
            preceding = Some(*first.data);
            let mut other = NativeTemplateLintContext::new(&linter, &foreign, "Foreign.vue");
            other.current_owner_profile = Profile::ClosedSelect;
            other.current_callback = Some(0);
            assert_eq!(
                other.first_option(&owner).err(),
                Some(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert_eq!(
                other.first_option_value(&owner, &first),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert_eq!(
                other.warn_first_option_with_help(&owner, &first, "message", "help"),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert!(other.finish().diagnostics.is_empty());
            assert!(context.finish().diagnostics.is_empty());
            seen += 1;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(seen, 2);
}

#[test]
fn empty_owner_needs_a_real_callback_and_later_refusal_never_issues_a_select_token() {
    let allocator = Allocator::new();
    for (source, expected) in [
        ("<select></select>", 1),
        (
            "<select><option value='a'>A</option><!--late--></select>",
            0,
        ),
    ] {
        let root = NativeLintComponent::parse_in(
            &allocator,
            SourceRoot::new(source).unwrap().whole_block(),
        )
        .unwrap();
        let linter = Linter::new();
        let mut seen = 0;
        let result = admission::children_with_owner_profile(
            root.component(),
            root.children(),
            NativeTemplateAttributeProfile::StaticOnly,
            NativeTemplateTextProfile::LiteralText,
            Profile::ClosedSelect,
            &mut |element| {
                let Some(owner) = element.closed_select_owner() else {
                    return Ok(());
                };
                let mut context = NativeTemplateLintContext::new(&linter, &root, "Empty.vue");
                context.current_owner_profile = Profile::ClosedSelect;
                assert_eq!(
                    context.first_option(&owner).err(),
                    Some(NativeTemplateLintRefusal::UnsupportedContext {
                        span: Span::new(0, 8)
                    })
                );
                context.current_callback = Some(0);
                assert!(context.first_option(&owner)?.is_none());
                seen += 1;
                Ok(())
            },
        );
        assert_eq!(seen, expected);
        assert_eq!(result.is_ok(), expected == 1);
    }
}
