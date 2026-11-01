use super::*;
use crate::native::template::NativeTemplateRule;
use crate::native::{header, template::NativeTemplateAttributeProfile as Profile};
use vize_l0::{Allocator, SourceRoot};

fn attributes<'o, 'a>(root: &'o NativeLintComponent<'a>) -> Vec<NativeTemplateAttribute<'o, 'a>> {
    let mut attributes = Vec::new();
    for child in root.children() {
        let Some(element) = child.into_element() else {
            continue;
        };
        for original in element.attributes() {
            let (checked, binding) = header::event_binding(&element, original).unwrap();
            attributes.push(
                NativeTemplateAttribute::from_checked(checked, binding, Profile::Events).unwrap(),
            );
        }
    }
    attributes
}

const MESSAGE: &str = "vue/no-deprecated-v-on-number-modifiers.message";
const HELP: &str = "vue/no-deprecated-v-on-number-modifiers.help";
const FIX: &str = "Replace the numeric keyCode modifier with a named key";
const RULE: &str = "vue/no-deprecated-v-on-number-modifiers";

#[test]
fn events_remain_a_separate_closed_profile_and_raw_tokens_are_unconstructible() {
    for a in [Profile::StaticOnly, Profile::Bindings, Profile::Events] {
        for b in [Profile::StaticOnly, Profile::Bindings, Profile::Events] {
            assert_eq!(a.intersect(b), if a == b { a } else { Profile::StaticOnly });
        }
    }
}

#[test]
fn genuine_modifier_tokens_retain_utf8_byte_ranges_order_and_exact_fixes() {
    let arena = Allocator::new();
    let source = "界🌸\r\n<input v-on:keyup.stop.13suffix.27=\"submit\"/>";
    let root =
        NativeLintComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
            .unwrap();
    let attributes = attributes(&root);
    let attribute = attributes.first().unwrap();
    let linter = crate::Linter::new();
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Token.vue");
    context.current_rule = RULE;
    context.current_callback = Some(0);
    let tokens = context
        .event_modifiers(attribute)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let expected: [(&str, Span); 3] = ["stop", "13suffix", "27"].map(|text| {
        let start = source.find(text).unwrap() as u32;
        (text, Span::new(start, start + text.len() as u32))
    });
    assert_eq!(
        tokens
            .iter()
            .map(|token| (token.text(), token.range))
            .collect::<Vec<_>>(),
        expected
    );
    let numeric = tokens
        .iter()
        .find(|token| token.text() == "13suffix")
        .unwrap();
    context
        .error_event_modifier_with_help(
            numeric,
            MESSAGE,
            &[("keyCode", numeric.text())],
            HELP,
            Some((FIX, "enter")),
        )
        .unwrap();
    let native = context.finish();
    let mut registry = crate::RuleRegistry::new();
    registry.register(Box::new(crate::rules::vue::NoDeprecatedVOnNumberModifiers));
    let legacy = crate::Linter::with_registry(registry).lint_template(source, "Token.vue");
    assert_eq!(format!("{native:#?}"), format!("{legacy:#?}"));
    let fix = native.diagnostics.first().unwrap().fix.as_ref().unwrap();
    assert_eq!(
        fix.apply(source).as_str(),
        "界🌸\r\n<input v-on:keyup.stop.enter.27=\"submit\"/>"
    );
}

#[test]
fn foreign_owner_sibling_forgery_and_other_callback_cannot_report_or_edit() {
    let arena = Allocator::new();
    let source = "<input @keyup.13=\"a\"/><input @keyup.27=\"b\"/>";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let root = NativeLintComponent::parse_in(&arena, block).unwrap();
    let foreign = NativeLintComponent::parse_in(&arena, block).unwrap();
    let proofs = attributes(&root);
    let foreign_proofs = attributes(&foreign);
    let linter = crate::Linter::new();
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Owned.vue");
    context.current_rule = RULE;
    context.current_callback = Some(0);
    assert_eq!(
        crate::rules::vue::NoDeprecatedVOnNumberModifiers.run_on_template(&mut context, &foreign),
        Err(NativeTemplateLintRefusal::SourceMismatch)
    );
    assert_eq!(
        context
            .event_modifiers(foreign_proofs.first().unwrap())
            .err()
            .unwrap(),
        NativeTemplateLintRefusal::SourceMismatch
    );
    let token = context
        .event_modifiers(proofs.first().unwrap())
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    context.current_callback = Some(1);
    assert_eq!(
        context.error_event_modifier_with_help(
            &token,
            MESSAGE,
            &[("keyCode", "13")],
            HELP,
            Some((FIX, "enter"))
        ),
        Err(NativeTemplateLintRefusal::SourceMismatch)
    );
    context.current_callback = Some(0);
    let sibling = NativeTemplateEventModifier {
        attribute: proofs.last().unwrap(),
        issuer: token.issuer,
        range: token.range,
        text: token.text,
    };
    assert_eq!(
        context.error_event_modifier_with_help(
            &sibling,
            MESSAGE,
            &[("keyCode", "13")],
            HELP,
            Some((FIX, "enter"))
        ),
        Err(NativeTemplateLintRefusal::SourceMismatch)
    );
    let changed_range = NativeTemplateEventModifier {
        attribute: token.attribute,
        issuer: token.issuer,
        range: Span::new(token.range.start, token.range.end + 1),
        text: token.text,
    };
    assert_eq!(
        context.error_event_modifier_with_help(
            &changed_range,
            MESSAGE,
            &[("keyCode", "13")],
            HELP,
            Some((FIX, "enter"))
        ),
        Err(NativeTemplateLintRefusal::SourceMismatch)
    );
    let other_linter = crate::Linter::new();
    let mut other = NativeTemplateLintContext::new(&other_linter, &root, "Other.vue");
    other.current_rule = RULE;
    other.current_callback = Some(0);
    assert_eq!(
        other.error_event_modifier_with_help(
            &token,
            MESSAGE,
            &[("keyCode", "13")],
            HELP,
            Some((FIX, "enter"))
        ),
        Err(NativeTemplateLintRefusal::SourceMismatch)
    );
    assert_eq!(
        format!("{:#?}", context.finish()),
        format!(
            "{:#?}",
            crate::LintResult {
                filename: "Owned.vue".into(),
                diagnostics: vec![],
                error_count: 0,
                warning_count: 0
            }
        )
    );
    assert_eq!(
        format!("{:#?}", other.finish()),
        format!(
            "{:#?}",
            crate::LintResult {
                filename: "Other.vue".into(),
                diagnostics: vec![],
                error_count: 0,
                warning_count: 0
            }
        )
    );
}
