use super::*;
use crate::native::template::NativeTemplateElement;
use crate::native::template::NativeTemplateRule;
use vize_l0::SmallVec;

fn checked<'o, 'a>(root: &'o NativeLintComponent<'a>) -> NativeTemplateElement<'o, 'a> {
    let element = root.children().next().unwrap().into_element().unwrap();
    let mut attributes = SmallVec::new();
    for original in element.attributes() {
        let (checked, binding) = header::wide_binding(&element, original).unwrap();
        attributes.push(
            NativeTemplateAttribute::from_checked(
                checked,
                binding,
                NativeTemplateAttributeProfile::Bindings,
            )
            .unwrap(),
        );
    }
    NativeTemplateElement::new(element, attributes)
}

#[test]
fn whole_void_header_reporter_preserves_original_range_and_all_configured_fields() {
    let allocator = Allocator::new();
    let source = "<input\r\n type='text' placeholder='雪🌸' />";
    let root =
        NativeLintComponent::parse_in(&allocator, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let view = checked(&root);
    let configured = crate::rules::a11y::FormControlHasLabel;
    let mut registry = crate::RuleRegistry::new();
    registry.register(Box::new(configured));
    let linter = Linter::with_registry(registry)
        .with_help_level(crate::HelpLevel::Full)
        .with_rule_severity_overrides(vec![(
            "a11y/form-control-has-label".into(),
            Severity::Error,
        )]);
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Exact.vue");
    context.current_rule = "a11y/form-control-has-label";
    context
        .warn_void_element_with_help(
            &view,
            "a11y/form-control-has-label.message",
            &[("tag", "input")],
            "a11y/form-control-has-label.help_placeholder",
        )
        .unwrap();
    let actual = context.finish();
    assert_eq!(
        format!("{actual:#?}"),
        format!("{:#?}", linter.lint_template(source, "Exact.vue"))
    );
    assert_eq!((actual.error_count, actual.warning_count), (1, 0));
    let diagnostic = actual.diagnostics.first().unwrap();
    assert_eq!(
        (diagnostic.start, diagnostic.end),
        (0, u32::try_from(source.len()).unwrap())
    );
}

#[test]
fn equal_source_foreign_owner_and_nonvoid_extent_cannot_mint_a_whole_void_finding() {
    let allocator = Allocator::new();
    for source in ["<input type='text'>", "<div></div>"] {
        let block = SourceRoot::new(source).unwrap().whole_block();
        let root = NativeLintComponent::parse_in(&allocator, block).unwrap();
        let foreign = NativeLintComponent::parse_in(&allocator, block).unwrap();
        let foreign_view = checked(&foreign);
        let linter = Linter::new();
        let mut context = NativeTemplateLintContext::new(&linter, &root, "Exact.vue");
        let callback = crate::rules::a11y::FormControlHasLabel;
        assert_eq!(
            callback.run_on_template(&mut context, &foreign),
            Err(NativeTemplateLintRefusal::SourceMismatch)
        );
        assert_eq!(
            callback.run_on_element(&mut context, &foreign_view),
            Err(NativeTemplateLintRefusal::SourceMismatch)
        );
        assert_eq!(
            context.element_opening_range(&foreign_view),
            Err(NativeTemplateLintRefusal::SourceMismatch)
        );
        assert_eq!(
            context.warn_void_element_with_help(&foreign_view, "message", &[], "help"),
            Err(NativeTemplateLintRefusal::SourceMismatch)
        );
        assert!(context.finish().diagnostics.is_empty());
        if source == "<div></div>" {
            let view = checked(&root);
            let mut context = NativeTemplateLintContext::new(&linter, &root, "Exact.vue");
            assert_eq!(
                context.warn_void_element_with_help(&view, "message", &[], "help"),
                Err(NativeTemplateLintRefusal::UnsupportedContext {
                    span: Span::new(0, 5)
                })
            );
            assert!(context.finish().diagnostics.is_empty());
        }
    }
}

#[test]
fn authenticated_self_closed_nonvoid_header_has_a_complete_original_extent() {
    // ElementClose::NotExpected includes authored self-closing tags as well
    // as HTML void tags (vize_l1::build::tag). The rule still selects input.
    let allocator = Allocator::new();
    let source = "<div title='exact'/>";
    let root =
        NativeLintComponent::parse_in(&allocator, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let view = checked(&root);
    assert!(matches!(
        view.original().surface().close,
        vize_l1::markup::ElementClose::NotExpected
    ));
    let linter = Linter::new().with_help_level(crate::HelpLevel::Full);
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Extent.vue");
    context.current_rule = "a11y/form-control-has-label";
    context
        .warn_void_element_with_help(
            &view,
            "a11y/form-control-has-label.message",
            &[("tag", "div")],
            "a11y/form-control-has-label.help",
        )
        .unwrap();
    let result = context.finish();
    assert_eq!((result.error_count, result.warning_count), (0, 1));
    let diagnostic = result.diagnostics.first().unwrap();
    assert_eq!(
        (diagnostic.start, diagnostic.end),
        (0, u32::try_from(source.len()).unwrap())
    );
    assert_eq!(
        diagnostic.message,
        "<div> elements must have an associated label"
    );
    assert!(diagnostic.labels.is_empty());
    assert!(diagnostic.fix.is_none());
    assert!(diagnostic.help.is_some());
}
