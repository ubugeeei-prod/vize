//! Whole native/component responses retain authored props, edits and defaults.

use serde_json::{Value, json};

use super::{ARIA, DATA, controls_project::Project, position, source_digest, with_edit};

const DECLARED: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/Declared.vue.txt"
);
const OWNER: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/Owner.vue.txt"
);
const PROPS: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/declared-props.expected.json"
);
const DEFAULTS: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/defaults.expected.json"
);

pub(super) fn assert_authored_hashes(manifest: &Value) {
    for (name, source) in [
        ("Declared.vue.txt", DECLARED),
        ("Owner.vue.txt", OWNER),
        ("declared-props.expected.json", PROPS),
        ("defaults.expected.json", DEFAULTS),
        (
            "declared-native.expected.json",
            include_str!(
                "../../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/declared-native.expected.json"
            ),
        ),
    ] {
        assert_eq!(
            source_digest(source),
            manifest["authoredControls"]["sha256"][name]
                .as_str()
                .unwrap()
        );
    }
}

fn declared_items(project: &Project, source: &str, dynamic: bool) -> Vec<Value> {
    let props: Value = serde_json::from_str(PROPS).unwrap();
    let mut items = props[if dynamic { "dynamic" } else { "static" }]
        .as_array()
        .unwrap()
        .clone();
    bind_native(project, source, &mut items);
    items
}

fn bind_native(project: &Project, source: &str, items: &mut [Value]) {
    for item in items {
        let name = match item["label"].as_str().unwrap() {
            "data-role" | "dataRole" => "dataRole",
            "aria-hidden" | "ariaHidden" => "ariaHidden",
            _ => continue,
        };
        if let Some(mut data) = project.declared_data(source, name) {
            data["vizeCompletion"]["label"] = item["label"].clone();
            item["data"] = data;
        }
    }
}

pub(super) fn edited_bank(source: &str, token: &str, cursor: &str, items: Vec<Value>) -> Value {
    let start = position(source, token);
    let end = position(source, cursor);
    json!(
        items
            .into_iter()
            .map(|item| {
                let insert = item["insertText"].as_str().unwrap();
                // These frozen banks preserve the existing colon/v-bind replacement
                // behavior, including the original common `:key` insertion.
                let insert = if token.starts_with(':')
                    && !insert.starts_with(':')
                    && !insert.starts_with("v-bind:")
                {
                    format!(":{insert}")
                } else if token.starts_with("v-bind:")
                    && !insert.starts_with(':')
                    && !insert.starts_with("v-bind:")
                {
                    format!("v-bind:{insert}")
                } else {
                    insert.to_owned()
                };
                with_edit(item, start.clone(), end.clone(), &insert)
            })
            .collect::<Vec<_>>()
    )
}

pub(super) fn changed(tag: &str, token: &str) -> String {
    // The astral character is before the requested attribute on the SAME line.
    OWNER
        .replace("<Declared  />", &format!("😀<{tag} {token} />"))
        .replace('\n', "\r\n")
}

#[test]
fn native_and_component_defaults_preserve_every_original_candidate_in_both_modes() {
    let defaults: Value = serde_json::from_str(DEFAULTS).unwrap();
    for native in [false, true] {
        let Some(mut project) = Project::new(OWNER, DECLARED, native) else {
            return;
        };
        for tag in ["Declared", "div"] {
            for prefix in ["", ":", "v-bind:"] {
                let source = changed(tag, prefix);
                project.change(&source);
                let mut items = defaults[if tag == "Declared" {
                    "component"
                } else {
                    "native"
                }][if prefix.is_empty() { "bare" } else { "bound" }]
                .as_array()
                .unwrap()
                .clone();
                if tag == "Declared" {
                    bind_native(&project, &source, &mut items);
                }
                let expected = if prefix.is_empty() {
                    json!(items)
                } else {
                    edited_bank(&source, &format!("{prefix} />"), " />", items)
                };
                project.assert_response(&source, " />", expected);
                project.assert_disk(OWNER, DECLARED);
            }
        }
        project.change(OWNER);
        let mut restored = defaults["component"]["bare"].as_array().unwrap().clone();
        bind_native(&project, OWNER, &mut restored);
        project.assert_response(OWNER, " />", json!(restored));
        project.assert_disk(OWNER, DECLARED);
        project.shutdown();
    }
}

#[test]
fn contextual_families_keep_declared_types_without_fallback_duplicates_in_both_modes() {
    let data: Value = serde_json::from_str(DATA).unwrap();
    let aria: Vec<Value> = serde_json::from_str(ARIA).unwrap();
    let defaults: Value = serde_json::from_str(DEFAULTS).unwrap();
    let aria_label = defaults["native"]["bare"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["label"] == "aria-label")
        .unwrap()
        .clone();
    for native in [false, true] {
        let Some(mut project) = Project::new(OWNER, DECLARED, native) else {
            return;
        };
        for tag in ["Declared", "div"] {
            for family in ["data-", "aria-"] {
                let source = changed(tag, family);
                project.change(&source);
                let mut items = if family == "data-" {
                    vec![data["data-*"].clone()]
                } else {
                    std::iter::once(aria_label.clone())
                        .chain(
                            aria.iter()
                                .filter(|item| tag != "Declared" || item["label"] != "aria-hidden")
                                .cloned(),
                        )
                        .collect()
                };
                if tag == "Declared" {
                    let props = declared_items(&project, &source, false);
                    items.push(props[if family == "data-" { 0 } else { 1 }].clone());
                }
                project.assert_response(
                    &source,
                    " />",
                    edited_bank(&source, &format!("{family} />"), " />", items),
                );
                project.assert_disk(OWNER, DECLARED);
            }
        }
        project.shutdown();
    }
}

#[test]
fn complete_static_and_bound_prop_payloads_keep_camel_kebab_authority_in_both_modes() {
    let defaults: Value = serde_json::from_str(DEFAULTS).unwrap();
    for native in [false, true] {
        let Some(mut project) = Project::new(OWNER, DECLARED, native) else {
            return;
        };
        for (token, index) in [
            ("data-role", 0),
            ("aria-hidden", 1),
            (":data-role", 0),
            (":aria-hidden", 1),
            ("v-bind:data-role", 0),
            ("v-bind:aria-hidden", 1),
        ] {
            let source = changed("Declared", token);
            project.change(&source);
            let items = if token.starts_with(':') || token.starts_with("v-bind:") {
                let mut items = defaults["component"]["bound"].as_array().unwrap().clone();
                bind_native(&project, &source, &mut items);
                items
            } else {
                vec![declared_items(&project, &source, false)[index].clone()]
            };
            let expected = edited_bank(&source, &format!("{token} />"), " />", items);
            project.assert_response(&source, " />", expected);
            project.assert_disk(OWNER, DECLARED);
        }
        project.shutdown();
    }
}
