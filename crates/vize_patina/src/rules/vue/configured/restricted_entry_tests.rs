//! Object entries of `vue/no-restricted-html-elements` and
//! `vue/no-restricted-v-on`, read through `linter.ruleOptions`.

use super::tests::reports;
use vize_l0::config::ConfigLintRuleOptions;

const ELEMENTS: &str = "vue/no-restricted-html-elements";
const V_ON: &str = "vue/no-restricted-v-on";

type Observed = Vec<(u32, u32, String)>;

/// `(line, column, message)` for the reports of `rule` configured as `options`.
fn observed(rule: &str, options: serde_json::Value, source: &str) -> Observed {
    let parsed: ConfigLintRuleOptions =
        serde_json::from_value(serde_json::json!({ rule: options })).unwrap();
    reports(source, &parsed)
        .into_iter()
        .map(|(line, column, reported, message)| {
            assert_eq!(reported, rule);
            (line, column, message)
        })
        .collect()
}

fn at(line: u32, column: u32, message: &str) -> (u32, u32, String) {
    (line, column, message.to_owned())
}

/// The reproduction of #13536.
const ANCHOR: &str = "<template>\n  <a />\n</template>\n";

#[test]
fn an_element_entry_reports_its_own_message_and_a_name_keeps_the_default() {
    let default = "Unexpected use of forbidden element a.";
    for (options, message) in [
        // The reproduction, then each row of its isolation table.
        (
            serde_json::json!([{ "element": ["a"], "message": "Use Bar." }]),
            "Use Bar.",
        ),
        (serde_json::json!([{ "element": ["a"] }]), default),
        (
            serde_json::json!([{ "element": "a", "message": "Use Bar." }]),
            "Use Bar.",
        ),
        (serde_json::json!(["a"]), default),
        (serde_json::json!("a"), default),
        // eslint-plugin-vue reads an empty message as no message.
        (
            serde_json::json!([{ "element": "a", "message": "" }]),
            default,
        ),
    ] {
        assert_eq!(
            observed(ELEMENTS, options.clone(), ANCHOR),
            [at(2, 3, message)],
            "{options}"
        );
    }
}

#[test]
fn mixed_element_entries_report_each_element_by_its_first_entry() {
    let source = "<template>\n  <marquee />\n  <a />\n  <button type=\"button\" />\n  <blink />\n  <p />\n</template>\n";
    assert_eq!(
        observed(
            ELEMENTS,
            serde_json::json!([
                "marquee",
                { "element": ["a", "button"], "message": "Use the design system." },
                { "element": "a", "message": "Never reached." },
                { "element": ["blink"] },
                "button",
            ]),
            source,
        ),
        [
            at(2, 3, "Unexpected use of forbidden element marquee."),
            at(3, 3, "Use the design system."),
            at(4, 3, "Use the design system."),
            at(5, 3, "Unexpected use of forbidden element blink."),
        ]
    );
    assert_eq!(
        observed(ELEMENTS, serde_json::json!([]), source),
        Observed::new()
    );
    assert_eq!(
        observed(
            ELEMENTS,
            serde_json::json!([{ "element": [], "message": "Nothing is named." }]),
            source
        ),
        Observed::new()
    );
}

const LISTENERS: &str = "<template>\n  <div @click.stop=\"a\" />\n  <div @click=\"b\" />\n  <div v-on:click.prevent.stop=\"c\" />\n  <MyInput @focus=\"d\" @click.stop=\"e\" />\n  <input @focus=\"f\" @[name].stop=\"g\" />\n</template>\n";

#[test]
fn a_listener_entry_restricts_the_event_with_its_modifiers() {
    let default = "Using `@click.stop` is not allowed on this element.";
    // The entry of #13534: `@click.stop`, with or without other modifiers.
    assert_eq!(
        observed(
            V_ON,
            serde_json::json!([{ "argument": "click", "modifiers": ["stop"] }]),
            LISTENERS
        ),
        [at(2, 8, default), at(4, 8, default), at(5, 23, default)]
    );
    assert_eq!(
        observed(
            V_ON,
            serde_json::json!([{
                "argument": "click",
                "modifiers": ["stop", "prevent"],
                "message": "Handle the event in the component.",
            }]),
            LISTENERS
        ),
        [at(4, 8, "Handle the event in the component.")]
    );
    assert_eq!(
        observed(
            V_ON,
            serde_json::json!([{ "argument": "click", "modifiers": ["once"] }]),
            LISTENERS
        ),
        Observed::new()
    );
}

#[test]
fn a_name_keeps_its_report_and_the_first_matching_entry_decides() {
    let named = "Using `@click` is not allowed on this element.";
    for options in [
        serde_json::json!("click"),
        serde_json::json!(["click"]),
        serde_json::json!([{ "argument": "click" }]),
        serde_json::json!([{ "argument": "click", "modifiers": [], "message": "" }]),
    ] {
        assert_eq!(
            observed(V_ON, options.clone(), LISTENERS),
            [
                at(2, 8, named),
                at(3, 8, named),
                at(4, 8, named),
                at(5, 23, named)
            ],
            "{options}"
        );
    }
    assert_eq!(
        observed(
            V_ON,
            serde_json::json!([
                { "argument": "click", "modifiers": ["stop"], "message": "No stop." },
                "click",
                { "argument": "focus", "element": "MyInput" },
            ]),
            LISTENERS
        ),
        [
            at(2, 8, "No stop."),
            at(3, 8, named),
            at(4, 8, "No stop."),
            at(5, 12, "Using `@focus` is not allowed on this <MyInput>."),
            at(5, 23, "No stop."),
        ]
    );
}
