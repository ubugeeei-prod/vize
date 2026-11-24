#![cfg(test)]
//! `vize lint` reads the option shapes of #13535, #13536, #13537 and #13534
//! from `vite.config` `vue.linter.ruleOptions`, written as their reporters
//! wrote them.

use crate::lint_rule_options_casing_cli::{output_details, temp_project_dir, write_project_file};
use std::{fs, path::PathBuf, process::Command};
use vize_l0::{String, cstr};

/// Exit code, the `(line, column, message)` reports of `rule`, and the first
/// `TypeError` the config runtime printed.
struct Lint {
    code: Option<i32>,
    reports: Vec<(u64, u64, String)>,
    refusal: Option<String>,
    project: PathBuf,
}

/// Lint `Foo.vue` in a project whose `vite.config.mjs` is the reporter's one
/// line: the rule as an error and `options` as its `ruleOptions` entry.
fn lint(rule: &str, options: &str, foo_vue: &str, arguments: &[&str]) -> Lint {
    let project = temp_project_dir();
    write_project_file(
        &project,
        "vite.config.mjs",
        cstr!(
            "export default {{ vue: {{ linter: {{ rules: {{ \"{rule}\": \"error\" }}, ruleOptions: {{ \"{rule}\": {options} }} }} }} }};\n"
        )
        .as_str(),
    );
    write_project_file(&project, "Foo.vue", foo_vue);
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project)
        .args(["lint", "--format", "json"])
        .args(arguments)
        .arg("Foo.vue")
        .output()
        .unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let refusal = stderr
        .lines()
        .find(|line| line.starts_with("TypeError: "))
        .map(String::from);
    let mut reports = Vec::new();
    if refusal.is_none() {
        let files: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{error}\n{}", output_details(&output)));
        for file in files.as_array().unwrap() {
            for message in file["messages"].as_array().unwrap() {
                if message["ruleId"] == rule {
                    reports.push((
                        message["line"].as_u64().unwrap(),
                        message["column"].as_u64().unwrap(),
                        String::from(message["message"].as_str().unwrap()),
                    ));
                }
            }
        }
    }
    Lint {
        code: output.status.code(),
        reports,
        refusal,
        project,
    }
}

/// The reports of one run, after removing its project.
fn observed(rule: &str, options: &str, foo_vue: &str) -> (Option<i32>, Vec<(u64, u64, String)>) {
    let lint = lint(rule, options, foo_vue, &[]);
    assert_eq!(lint.refusal, None);
    let _ = fs::remove_dir_all(&lint.project);
    (lint.code, lint.reports)
}

fn refused(rule: &str, options: &str) -> (Option<i32>, Option<String>) {
    let lint = lint(rule, options, "<template>\n  <div />\n</template>\n", &[]);
    let _ = fs::remove_dir_all(&lint.project);
    (lint.code, lint.refusal)
}

fn report(line: u64, column: u64, message: &str) -> (u64, u64, String) {
    (line, column, String::from(message))
}

const SAME_NAME: &str = "<template>\n  <div :foo=\"foo\" />\n</template>\n";

#[test]
fn v_bind_style_reads_the_upstream_tuple_and_reports_same_name_bindings() {
    let rule = "vue/v-bind-style";
    // The reproduction of #13535, then the rows of its isolation table.
    assert_eq!(
        observed(
            rule,
            r#"["shorthand", { sameNameShorthand: "always" }]"#,
            SAME_NAME
        ),
        (
            Some(1),
            vec![report(
                2,
                8,
                "[vize:vue/v-bind-style] Expected same-name shorthand."
            )]
        )
    );
    assert_eq!(
        observed(rule, r#"["shorthand"]"#, SAME_NAME),
        (Some(0), vec![])
    );
    assert_eq!(
        observed(rule, r#""shorthand""#, SAME_NAME),
        (Some(0), vec![])
    );
    assert_eq!(
        observed(
            rule,
            r#""shorthand""#,
            "<template>\n  <div :foo=\"foo\" />\n  <div v-bind:bar=\"bar\" />\n</template>\n"
        ),
        (
            Some(1),
            vec![report(
                3,
                8,
                "[vize:vue/v-bind-style] Prefer shorthand `:` over `v-bind:`"
            )]
        )
    );
    // `never` was not run by the reporter.
    assert_eq!(
        observed(
            rule,
            r#"["shorthand", { sameNameShorthand: "never" }]"#,
            "<template>\n  <div :foo-bar :baz=\"baz\" />\n</template>\n"
        ),
        (
            Some(1),
            vec![report(
                2,
                8,
                "[vize:vue/v-bind-style] Unexpected same-name shorthand."
            )]
        )
    );
}

#[test]
fn v_bind_style_fixes_both_same_name_policies_and_refuses_other_shapes() {
    let rule = "vue/v-bind-style";
    for (options, before, after) in [
        (
            r#"["shorthand", { sameNameShorthand: "always" }]"#,
            "<template>\n  <div v-bind:foo=\"foo\" :bar-baz.camel=\"barBaz\" :qux=\"other\" />\n</template>\n",
            "<template>\n  <div :foo :bar-baz.camel :qux=\"other\" />\n</template>\n",
        ),
        (
            r#"["longform", { sameNameShorthand: "never" }]"#,
            "<template>\n  <div :foo .bar-baz v-bind:qux=\"other\" />\n</template>\n",
            "<template>\n  <div v-bind:foo=\"foo\" v-bind:bar-baz.prop=\"barBaz\" v-bind:qux=\"other\" />\n</template>\n",
        ),
    ] {
        let fixed = lint(rule, options, before, &["--fix"]);
        assert_eq!(fixed.refusal, None, "{options}");
        assert_eq!(
            fs::read_to_string(fixed.project.join("Foo.vue")).unwrap(),
            after,
            "{options}"
        );
        assert_eq!((fixed.code, fixed.reports), (Some(0), vec![]), "{options}");
        let _ = fs::remove_dir_all(&fixed.project);
    }

    let one_type = "TypeError: vue.linter.ruleOptions.vue/v-bind-style: must match one documented native option type";
    for options in [
        r#"["shorthand", { sameNameShorthand: "sometimes" }]"#,
        r#"["shorthand", { sameName: "always" }]"#,
        r#"["shorthand", "longform"]"#,
        r#"["shorthand", {}, {}]"#,
        r#"[{ sameNameShorthand: "always" }]"#,
        r#"{ sameNameShorthand: "always" }"#,
    ] {
        assert_eq!(
            refused(rule, options),
            (Some(2), Some(String::from(one_type))),
            "{options}"
        );
    }
}

const ANCHOR: &str = "<template>\n  <a />\n</template>\n";

#[test]
fn no_restricted_html_elements_reports_the_message_of_its_entry() {
    let rule = "vue/no-restricted-html-elements";
    let default = "[vize:vue/no-restricted-html-elements] Unexpected use of forbidden element a.";
    let custom = "[vize:vue/no-restricted-html-elements] Use Bar.";
    // The reproduction of #13536, then the rows of its isolation table.
    for (options, message) in [
        (r#"[{ element: ["a"], message: "Use Bar." }]"#, custom),
        (r#"[{ element: ["a"] }]"#, default),
        (r#"[{ element: "a", message: "Use Bar." }]"#, custom),
        (r#"["a"]"#, default),
        (r#""a""#, default),
    ] {
        assert_eq!(
            observed(rule, options, ANCHOR),
            (Some(1), vec![report(2, 3, message)]),
            "{options}"
        );
    }
    assert_eq!(
        observed(
            rule,
            r#"["marquee", { element: ["a", "button"], message: "Use the design system." }]"#,
            "<template>\n  <marquee />\n  <a />\n  <p />\n  <button type=\"button\" />\n</template>\n"
        ),
        (
            Some(1),
            vec![
                report(
                    2,
                    3,
                    "[vize:vue/no-restricted-html-elements] Unexpected use of forbidden element marquee."
                ),
                report(
                    3,
                    3,
                    "[vize:vue/no-restricted-html-elements] Use the design system."
                ),
                report(
                    5,
                    3,
                    "[vize:vue/no-restricted-html-elements] Use the design system."
                ),
            ]
        )
    );
    let one_type = "TypeError: vue.linter.ruleOptions.vue/no-restricted-html-elements: must match one documented native option type";
    for options in [
        r#"[{ message: "Use Bar." }]"#,
        r#"[{ element: "a", msg: "Use Bar." }]"#,
        r#"{ element: "a" }"#,
    ] {
        assert_eq!(
            refused(rule, options),
            (Some(2), Some(String::from(one_type))),
            "{options}"
        );
    }
}

const PRIMITIVE: &str = "<template>\n  <primitive />\n</template>\n";

#[test]
fn component_name_in_template_casing_exempts_the_listed_names() {
    let rule = "vue/component-name-in-template-casing";
    let pascal = "[vize:vue/component-name-in-template-casing] Component should use PascalCase";
    // The reproduction of #13537 with the option it asks for, then its rows.
    for (options, source, reports) in [
        (
            r#"{ ignores: ["primitive"], registeredComponentsOnly: false }"#,
            PRIMITIVE,
            vec![],
        ),
        (
            "{ registeredComponentsOnly: false }",
            PRIMITIVE,
            vec![report(2, 3, pascal)],
        ),
        (
            r#"{ globals: ["primitive"], registeredComponentsOnly: false }"#,
            PRIMITIVE,
            vec![report(2, 3, pascal)],
        ),
        ("{ registeredComponentsOnly: true }", PRIMITIVE, vec![]),
        (
            r#"{ globals: ["primitive"], registeredComponentsOnly: true }"#,
            PRIMITIVE,
            vec![report(2, 3, pascal)],
        ),
        (
            r#"{ globals: ["primitive"], ignores: ["primitive"], registeredComponentsOnly: true }"#,
            PRIMITIVE,
            vec![],
        ),
        (
            r#"{ ignores: ["primitive"], registeredComponentsOnly: false }"#,
            "<template>\n  <primitive />\n  <foo-bar />\n</template>\n",
            vec![report(3, 3, pascal)],
        ),
        (
            r#"{ ignores: ["/^(primitive|foo-)/"], registeredComponentsOnly: false }"#,
            "<template>\n  <primitive />\n  <foo-bar />\n  <bar-foo />\n</template>\n",
            vec![report(4, 3, pascal)],
        ),
    ] {
        let code = if reports.is_empty() { 0 } else { 1 };
        assert_eq!(
            observed(rule, options, source),
            (Some(code), reports),
            "{options}"
        );
    }
    assert_eq!(
        refused(rule, r#"{ ignores: ["primitive", "primitive"] }"#),
        (
            Some(2),
            Some(String::from(
                "TypeError: vue.linter.ruleOptions.vue/component-name-in-template-casing.ignores: must contain unique items"
            ))
        )
    );
    assert_eq!(
        refused(rule, r#"{ ignore: ["primitive"] }"#),
        (
            Some(2),
            Some(String::from(
                "TypeError: vue.linter.ruleOptions.vue/component-name-in-template-casing: unknown option \"ignore\""
            ))
        )
    );
}

#[test]
fn no_restricted_v_on_reads_the_upstream_object_entry() {
    let rule = "vue/no-restricted-v-on";
    // The entry of #13534: it was refused at config load.
    assert_eq!(
        observed(
            rule,
            r#"[{ argument: "click", modifiers: ["stop"] }]"#,
            "<template>\n  <div @click.stop />\n  <div @click=\"a\" />\n  <div v-on:click.stop.prevent=\"b\" />\n</template>\n"
        ),
        (
            Some(1),
            vec![
                report(
                    2,
                    8,
                    "[vize:vue/no-restricted-v-on] Using `@click.stop` is not allowed on this element."
                ),
                report(
                    4,
                    8,
                    "[vize:vue/no-restricted-v-on] Using `@click.stop` is not allowed on this element."
                ),
            ]
        )
    );
    assert_eq!(
        observed(
            rule,
            r#"[{ argument: "click", modifiers: ["stop"], message: "Do not stop clicks." }, "focus"]"#,
            "<template>\n  <div @click.stop=\"a\" @focus=\"b\" />\n</template>\n"
        ),
        (
            Some(1),
            vec![
                report(2, 8, "[vize:vue/no-restricted-v-on] Do not stop clicks."),
                report(
                    2,
                    24,
                    "[vize:vue/no-restricted-v-on] Using `@focus` is not allowed on this element."
                ),
            ]
        )
    );
    let one_type = "TypeError: vue.linter.ruleOptions.vue/no-restricted-v-on: must match one documented native option type";
    for options in [
        r#"[{ modifiers: ["stop"] }]"#,
        "[{ argument: null }]",
        "[null]",
    ] {
        assert_eq!(
            refused(rule, options),
            (Some(2), Some(String::from(one_type))),
            "{options}"
        );
    }
}
