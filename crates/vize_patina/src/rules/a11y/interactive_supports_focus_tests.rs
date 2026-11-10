use super::InteractiveSupportsFocus;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::output::{OutputFormat, format_results};
use crate::rule::RuleRegistry;
use serde_json::Value;
use vize_atelier_jsx::JsxLang;

const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/a11y-listbox-focus-10925/cases.json"
));

fn linter(legacy: bool) -> Linter {
    if legacy {
        return super::interactive_supports_focus::create_legacy_linter_for_test();
    }
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(InteractiveSupportsFocus));
    Linter::with_registry(registry)
}

fn packet(result: LintResult, source: &str, filename: &str) -> Value {
    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.rule_name, "a11y/interactive-supports-focus");
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert!(diagnostic.help.is_some());
        assert!(diagnostic.labels.is_empty());
        assert!(diagnostic.fix.is_none());
    }
    serde_json::from_str(&format_results(
        &[result],
        &[(filename.into(), source.into())],
        OutputFormat::Json,
    ))
    .expect("complete lint JSON")
}

fn current_expected_packet(route: &Value) -> Value {
    let mut expected = route["afterPacket"].clone();
    for file in expected.as_array_mut().expect("complete pinned report") {
        for message in file["messages"].as_array_mut().expect("complete messages") {
            assert_eq!(message["ruleId"], "a11y/interactive-supports-focus");
            assert_eq!(
                message["ruleDocsPath"],
                "docs/content/rules/accessibility.md"
            );
            message["ruleDocsPath"] =
                "docs/content/rules/reference/a11y-interactive-supports-focus.md".into();
        }
    }
    expected
}

#[test]
fn current_documentation_oracle_rejects_invalid_frozen_urls_and_foreign_rules() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let case = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "empty-listbox")
        .expect("the authored single-warning control");
    let mut route = case["routes"]["vue"].clone();
    route["afterPacket"] = route["beforePacket"].clone();
    let expected = current_expected_packet(&route);
    assert_eq!(expected[0]["messages"].as_array().unwrap().len(), 1);
    for page in [
        "docs/content/rules/vue.md",
        "docs/content/rules/reference/missing-page.md",
        "docs/content/rules/reference/a11y-alt-text.md",
    ] {
        let mut invalid = route.clone();
        invalid["afterPacket"][0]["messages"][0]["ruleDocsPath"] = page.into();
        std::panic::catch_unwind(|| current_expected_packet(&invalid))
            .expect_err("invalid frozen documentation must fail the owned projection");
    }
    let mut foreign = route;
    foreign["afterPacket"][0]["messages"][0]["ruleId"] = "a11y/alt-text".into();
    std::panic::catch_unwind(|| current_expected_packet(&foreign))
        .expect_err("the pinned consumer owns exactly its declared rule");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for locale in ["", "ja/"] {
        let page =
            format!("docs/content/{locale}rules/reference/a11y-interactive-supports-focus.md");
        assert!(root.join(&page).is_file(), "{page}");
    }
}

#[test]
fn original_composite_widgets_keeps_children_as_the_focus_owner() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/linter/a11y-listbox-focus-10925/originalCompositeWidgets.vue.txt"
    ));
    for legacy in [false, true] {
        let linter = linter(legacy);
        for newline in ["\n", "\r\n"] {
            let source = source.replace('\n', newline);
            let result = linter.lint_sfc(&source, "CompositeWidgets.vue");
            assert_eq!(result.error_count, 0);
            assert_eq!(result.warning_count, 0, "{legacy}/{newline:?}: {result:?}");
            assert!(result.diagnostics.is_empty());
        }
    }
}

#[test]
fn listbox_ownership_controls_keep_complete_packets_on_native_and_legacy_hosts() {
    let corpus: Value = serde_json::from_str(CASES).expect("pinned listbox corpus");
    let cases = corpus["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 37);
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/linter/a11y-listbox-focus-10925");
    let native = linter(false);
    let legacy = linter(true);
    for case in cases {
        for route_name in ["vue", "html", "jsx", "tsx"] {
            let route = &case["routes"][route_name];
            let source = std::fs::read_to_string(
                directory.join(route["sourceFile"].as_str().expect("source file")),
            )
            .expect("whole source");
            let filename = route["filename"].as_str().expect("filename");
            for repetition in 0..2 {
                let result = match route_name {
                    "vue" => native.lint_sfc(&source, filename),
                    "html" => native.lint_standalone_html(&source, filename),
                    "jsx" => native.lint_jsx(&source, filename, JsxLang::Jsx),
                    "tsx" => native.lint_jsx(&source, filename, JsxLang::Tsx),
                    _ => unreachable!(),
                };
                assert_eq!(
                    packet(result, &source, filename),
                    current_expected_packet(route),
                    "{}/{route_name}/{repetition}",
                    case["id"]
                );
                if matches!(route_name, "vue" | "html") {
                    let result = if route_name == "vue" {
                        legacy.lint_sfc(&source, filename)
                    } else {
                        legacy.lint_template(&source, filename)
                    };
                    assert_eq!(
                        packet(result, &source, filename),
                        current_expected_packet(route),
                        "legacy {}/{route_name}/{repetition}",
                        case["id"]
                    );
                }
            }
        }
    }
}

#[test]
fn test_valid_activedescendant_listbox() {
    let linter = linter(false);
    let result = linter.lint_template(
        r#"<div>
  <button
type="button"
role="combobox"
aria-controls="lb"
:aria-activedescendant="`lb-option-${focused}`"
  >Select</button>
  <div id="lb" role="listbox">
<button role="option" tabindex="-1">A</button>
  </div>
</div>"#,
        "test.vue",
    );
    assert_eq!(result.warning_count, 0);
}

#[test]
fn combobox_listbox_and_options_need_no_focus() {
    let linter = linter(false);
    for ids in [
        r#"aria-controls="fruits" | id="fruits""#,
        r#":aria-controls="listId" | :id="listId""#,
        r#":aria-controls="'fruits'" | id="fruits""#,
        r#"aria-controls="other fruits" | :id="`fruits`""#,
    ] {
        let (controls, id) = ids.split_once(" | ").unwrap_or_default();
        let source = format!(
            r#"<input aria-activedescendant="fruit-0" {controls} aria-expanded="true" aria-label="Fruit" role="combobox" />
<ul {id} role="listbox">
  <li id="fruit-0" aria-selected="true" role="option">Apple</li>
  <li role="option"><span>Banana</span></li>
</ul>"#
        );
        let result = linter.lint_template(&source, "test.vue");
        assert_eq!(result.warning_count, 0, "{ids}: {:?}", result.diagnostics);
    }
}

#[test]
fn combobox_with_bound_ids_in_an_sfc() {
    let source = r#"<script setup lang="ts">
const listId = "fruits";
</script>

<template>
  <input
aria-activedescendant="fruit-0"
:aria-controls="listId"
aria-expanded="true"
aria-label="Fruit"
role="combobox"
  />
  <ul :id="listId" role="listbox">
<li id="fruit-0" aria-selected="true" role="option">Apple</li>
  </ul>
</template>
"#;
    let result = linter(false).lint_sfc(source, "BoundIds.vue");
    assert_eq!(result.warning_count, 0, "{:?}", result.diagnostics);
}

#[test]
fn handlerless_listboxes_and_options_do_not_need_focus() {
    let linter = linter(false);
    for source in [
        // No aria-activedescendant on the controlling element.
        r#"<input :aria-controls="listId" role="combobox" /><ul :id="listId" role="listbox"><li role="option">A</li></ul>"#,
        // A different expression is a different id.
        r#"<input aria-activedescendant="a" :aria-controls="listId" role="combobox" /><ul :id="otherId" role="listbox"><li role="option">A</li></ul>"#,
    ] {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(
            result.warning_count, 0,
            "{source}: {:?}",
            result.diagnostics
        );
    }
    // Options outside the controlled listbox are still checked.
    let outside = r#"<input aria-activedescendant="a" aria-controls="lb" role="combobox" /><ul id="lb" role="listbox"><li role="option">A</li></ul><div role="option">B</div>"#;
    assert_eq!(linter.lint_template(outside, "test.vue").warning_count, 0);
}

#[test]
fn matches_the_plugin_on_issue_12762_except_focusable_owners() {
    // The plugin also reports 45 and 46 (roving tabindex listboxes) and 50 and
    // 60, whose bound `tabindex` is an integer on every branch.
    let source = r#"<script setup lang="ts">
import MyOption from "./MyOption.vue";

const onEvent = (): void => undefined;
const keys = {
  onKeydown: (): void => undefined,
  tabindex: (option: string): number => (option === "a" ? 0 : -1),
};
const role = "button";
const enabled = true;
const hidden = true;
const label = "Options";
const options = ["a", "b"];
const selected = "a";
const select = (option: string): void => void option;
</script>

<template>
  <div role="button" @contextmenu="onEvent">x</div>
  <div role="button" @doubleclick="onEvent">x</div>
  <div role="button" @drag="onEvent">x</div>
  <div role="button" @dragend="onEvent">x</div>
  <div role="button" @dragenter="onEvent">x</div>
  <div role="button" @dragexit="onEvent">x</div>
  <div role="button" @dragleave="onEvent">x</div>
  <div role="button" @dragover="onEvent">x</div>
  <div role="button" @dragstart="onEvent">x</div>
  <div role="button" @drop="onEvent">x</div>
  <div role="button" @mouseenter="onEvent">x</div>
  <div role="button" @mouseleave="onEvent">x</div>
  <div role="button" @mousemove="onEvent">x</div>
  <div role="button" @mouseout="onEvent">x</div>
  <div role="button" @mouseover="onEvent">x</div>
  <div role="grid" @click="onEvent">x</div>
  <div role="menubar" @click="onEvent">x</div>
  <div role="progressbar" @click="onEvent">x</div>
  <div role="radiogroup" @click="onEvent">x</div>
  <div role="rowheader" @click="onEvent">x</div>
  <div role="treegrid" @click="onEvent">x</div>
  <div role="doc-backlink" @click="onEvent">x</div>
  <div role="doc-biblioref" @click="onEvent">x</div>
  <div role="doc-glossref" @click="onEvent">x</div>
  <div role="doc-noteref" @click="onEvent">x</div>
  <div role="listbox" :aria-label="label" class="rows" @keydown="keys.onKeydown"><MyOption v-for="option in options" :id="option" :key="option" :selected="option === selected" :tabindex="keys.tabindex(option)" @click="select(option)">{{ option }}</MyOption></div>
  <div role="listbox" :aria-label="label" @keydown="onEvent"><div role="option" tabindex="0" aria-selected="true">a</div></div>
  <div role="listbox" :aria-label="label" @keydown="onEvent"><div v-for="option in options" :key="option" role="option" :tabindex="option === selected ? 0 : -1" aria-selected="false">{{ option }}</div></div>
  <div role="button" aria-disabled="true" @click="onEvent">x</div>
  <div role="button" aria-hidden="true" @click="onEvent">x</div>
  <div role="button" :aria-hidden="hidden" @click="onEvent">x</div>
  <div role="button" :tabindex="enabled ? 0 : -1" @click="onEvent">x</div>
  <div role="button" :tabindex="enabled ? 0 : undefined" @click="onEvent">x</div>
  <div role="button" tabindex @click="onEvent">x</div>
  <div role="button" tabindex="" @click="onEvent">x</div>
  <div :role="'button'" @click="onEvent">x</div>
  <a role="button" @click="onEvent">x</a>
  <svg><polygon points="0,0 1,1 0,1" role="button" @click="onEvent" /></svg>
  <svg role="button" @click="onEvent"><path d="M0 0h1" /></svg>
  <div role="textbox" contenteditable="true" @keydown="onEvent">x</div>
  <div role="button" @click="keys.onKeydown">x</div>
  <div role="tab" :tabindex="selected === 'a' ? 0 : -1" :aria-selected="selected === 'a'" @click="select('a')">a</div>
  <div role="BUTTON" @click="onEvent">x</div>
  <div role="button link" @click="onEvent">x</div>
</template>
"#;
    let result = linter(false).lint_sfc(source, "App.vue");
    let lines: Vec<usize> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            source
                .get(..diagnostic.start as usize)
                .map_or(0, |prefix| prefix.matches('\n').count())
                + 1
        })
        .collect();
    assert_eq!(
        lines,
        vec![
            19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
            41, 42, 43, 51, 53, 54, 55, 58, 61, 62
        ],
        "{:?}",
        result.diagnostics
    );
}
