use super::ValidVOn;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVOn));
    Linter::with_registry(registry)
}

#[test]
fn test_valid_v_on_click() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div @click="handleClick"></div>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_on_long_form() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-on:click="handleClick"></div>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_valid_v_on_modifier_only() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<form @submit.prevent></form>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn test_invalid_v_on_no_handler() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div @click></div>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_on_keyword_handler() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div @click="for"></div>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_invalid_v_on_keyword_object_syntax() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div v-on="class"></div>"#, "test.vue");
    assert_eq!(result.error_count, 1);
}

#[test]
fn test_valid_v_on_literal_handler_expression() {
    let linter = create_linter();
    let result = linter.lint_template(r#"<div @click="true"></div>"#, "test.vue");
    assert_eq!(result.error_count, 0);
}

#[test]
fn unsupported_modifiers_report_each_exact_range() {
    for attr in [
        "@click.sync.nativeish",
        "v-on:click.sync.nativeish",
        "@[event].sync.nativeish",
        "v-on.sync.nativeish",
    ] {
        let source = vize_l0::cstr!(r#"<div {attr}="handler" />"#);
        let result = create_linter().lint_template(&source, "test.vue");
        assert_eq!(result.error_count, 2, "{source}");
        for (diagnostic, name) in result.diagnostics.iter().zip(["sync", "nativeish"]) {
            assert_eq!(diagnostic.rule_name, "vue/valid-v-on");
            assert_eq!(
                &source[diagnostic.start as usize..diagnostic.end as usize],
                name
            );
            assert!(diagnostic.message.contains(name));
        }
    }
}

#[test]
fn valid_keyboard_builtins_numeric_and_unicode_modifiers() {
    let linter = create_linter();
    for modifier in super::VALID_MODIFIERS
        .iter()
        .chain(super::key_aliases::KEY_ALIASES)
    {
        let source = vize_l0::cstr!(r#"<input @keyup.{modifier}="handler" />"#);
        assert_eq!(
            linter.lint_template(&source, "test.vue").error_count,
            0,
            "{modifier}"
        );
    }
    for modifier in [
        "13",
        "13suffix",
        "0x10",
        "-13suffix",
        "+13suffix",
        "あ",
        "😀",
    ] {
        let source = vize_l0::cstr!(r#"<input @keyup.{modifier}="handler" />"#);
        assert_eq!(
            linter.lint_template(&source, "test.vue").error_count,
            0,
            "{modifier}"
        );
    }
    assert!(
        super::key_aliases::KEY_ALIASES
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
}

#[test]
fn numeric_prefix_matches_javascript_parse_int() {
    for valid in [
        "13suffix",
        "-13suffix",
        "+13suffix",
        "0x10",
        "\u{feff}13",
        "\u{a0}-13",
        "0e999",
    ] {
        assert!(super::has_integer_prefix(valid), "{valid:?}");
    }
    for invalid in [
        "", "suffix13", "-", "+", "Infinity", "NaN", "１２", "\u{85}13",
    ] {
        assert!(!super::has_integer_prefix(invalid), "{invalid:?}");
    }
    assert!(super::has_integer_prefix(&"9".repeat(308)));
    assert!(!super::has_integer_prefix(&"9".repeat(309)));
}

#[test]
fn multi_codepoint_and_similarly_named_modifiers_are_invalid() {
    for modifier in [
        "sync",
        "nativeish",
        "entering",
        "pageDown",
        "😀😀",
        "e\u{301}",
    ] {
        let source = vize_l0::cstr!(r#"<input @keyup.{modifier}="handler" />"#);
        assert_eq!(
            create_linter()
                .lint_template(&source, "test.vue")
                .error_count,
            1,
            "{modifier}"
        );
    }
}

#[test]
fn only_verb_modifiers_allow_missing_handler() {
    for attr in [
        "@click.stop",
        "@submit.prevent",
        "@click.once.stop",
        "@keyup.enter.prevent",
    ] {
        let source = vize_l0::cstr!("<div {attr} />");
        assert_eq!(
            create_linter()
                .lint_template(&source, "test.vue")
                .error_count,
            0,
            "{source}"
        );
    }
    for attr in [
        "@click.once",
        "@keyup.enter",
        "@click.native",
        "@click.capture",
        "@click=\"\"",
        "@click.once=\"  \"",
    ] {
        let source = vize_l0::cstr!("<div {attr} />");
        assert_eq!(
            create_linter()
                .lint_template(&source, "test.vue")
                .error_count,
            1,
            "{source}"
        );
    }
    let result = create_linter().lint_template("<div @click.nativeish />", "test.vue");
    assert_eq!(result.error_count, 2);
}

#[test]
fn custom_modifiers_supplement_defaults_but_do_not_supply_handler() {
    let linter = create_linter().with_valid_v_on_modifiers(vec!["custom".into()]);
    assert_eq!(
        linter
            .lint_template(r#"<div @click.custom.stop="handler" />"#, "test.vue")
            .error_count,
        0
    );
    assert_eq!(
        linter
            .lint_template(r#"<div @click.Custom="handler" />"#, "test.vue")
            .error_count,
        1
    );
    assert_eq!(
        linter
            .lint_template("<div @click.custom />", "test.vue")
            .error_count,
        1
    );
    let linter = linter.with_valid_v_on_modifiers(vec![]);
    assert_eq!(
        linter
            .lint_template(r#"<div @click.custom="handler" />"#, "test.vue")
            .error_count,
        1
    );
    let linter =
        Linter::with_registry(RuleRegistry::new()).with_valid_v_on_modifiers(vec!["custom".into()]);
    assert!(!linter.registry().has_rule("vue/valid-v-on"));
}
