use super::{Allocator, NativeLintComponent, NativeTemplateLintContext, SourceRoot, attributes};
use crate::native::template::{NativeTemplateEventModifier, NativeTemplateLintRefusal as Refusal};
use crate::rules::vue::NoDeprecatedVOnNativeModifier;
use crate::{LintResult, Linter, RuleRegistry};

const RULE: &str = "vue/no-deprecated-v-on-native-modifier";
const MESSAGE: &str = "vue/no-deprecated-v-on-native-modifier.message";
const HELP: &str = "vue/no-deprecated-v-on-native-modifier.help";

fn rejected(scenario: &str, verdict: Result<(), Refusal>, actual: LintResult) {
    println!("authority_case:{scenario}\nverdict:{verdict:#?}\nwhole_result:{actual:#?}");
    assert_eq!(verdict, Err(Refusal::SourceMismatch));
    assert_eq!(
        format!("{actual:#?}"),
        format!(
            "{:#?}",
            LintResult {
                filename: "Token.vue".into(),
                diagnostics: vec![],
                error_count: 0,
                warning_count: 0,
            }
        )
    );
}

#[test]
fn fix_none_preserves_owner_callback_linter_and_sibling_token_authority() {
    let arena = Allocator::new();
    let source = "<div @click.native=\"a\"/><div @keyup.native=\"b\"/>";
    let block = SourceRoot::new(source).unwrap().whole_block();
    let root = NativeLintComponent::parse_in(&arena, block).unwrap();
    let foreign = NativeLintComponent::parse_in(&arena, block).unwrap();
    let proofs = attributes(&root);
    let foreign_proofs = attributes(&foreign);
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedVOnNativeModifier));
    let linter = Linter::with_registry(registry);
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Token.vue");
    context.current_rule = RULE;
    context.current_callback = Some(0);
    let verdict = context
        .event_modifiers(foreign_proofs.first().unwrap())
        .map(|_| ());
    rejected("foreign-owner", verdict, context.finish());
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Token.vue");
    context.current_rule = RULE;
    context.current_callback = Some(0);
    let token = context
        .event_modifiers(proofs.first().unwrap())
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    context.current_callback = Some(1);
    let verdict = context.error_event_modifier_with_help(&token, MESSAGE, &[], HELP, None);
    rejected("foreign-callback", verdict, context.finish());
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Token.vue");
    context.current_rule = RULE;
    context.current_callback = Some(0);
    let forged = NativeTemplateEventModifier {
        attribute: proofs.last().unwrap(),
        issuer: token.issuer,
        range: token.range,
        text: token.text,
    };
    let verdict = context.error_event_modifier_with_help(&forged, MESSAGE, &[], HELP, None);
    rejected("forged-sibling", verdict, context.finish());
    let other_linter = Linter::new();
    let mut other = NativeTemplateLintContext::new(&other_linter, &root, "Token.vue");
    other.current_rule = RULE;
    other.current_callback = Some(0);
    let verdict = other.error_event_modifier_with_help(&token, MESSAGE, &[], HELP, None);
    rejected("foreign-linter", verdict, other.finish());
    let mut context = NativeTemplateLintContext::new(&linter, &root, "Token.vue");
    context.current_rule = RULE;
    context.current_callback = Some(0);
    for attribute in &proofs {
        let modifier = context
            .event_modifiers(attribute)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        context
            .error_event_modifier_with_help(&modifier, MESSAGE, &[], HELP, None)
            .unwrap();
    }
    let actual = context.finish();
    let legacy = linter.lint_template(source, "Token.vue");
    println!("source:{source:?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
    assert_eq!(format!("{actual:#?}"), format!("{legacy:#?}"));
    assert!(
        actual
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.fix.is_none())
    );
}
