//! Execute the published examples through the public linter, including context.
use serde_json::Value;
use std::path::{Path, PathBuf};
use vize_l0::{String, cstr};
use vize_patina::{LintPreset, Linter};

mod public_expression_reports;

#[test]
fn documented_single_file_bad_and_good_examples_match_the_real_rules() {
    verify_examples(false);
}

#[test]
fn documented_type_aware_examples_match_with_the_required_corsa_runtime() {
    if std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() != Ok("1") {
        eprintln!("Type-aware documentation examples require the Corsa Actions lane");
        return;
    }
    verify_examples(true);
    public_expression_reports::verify_required_reports();
}

fn reference_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/content/rules/reference")
}

fn verify_examples(type_aware: bool) {
    let mut paths: Vec<_> = std::fs::read_dir(reference_root())
        .expect("generated rule reference directory")
        .map(|entry| entry.expect("reference entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 258, "all current rule implementations");
    let mut checked = 0;
    let mut unavailable = 0;
    let mut parser_owned = 0;
    let mut failures = Vec::new();
    for path in paths {
        let page = std::fs::read_to_string(&path).expect("reference page");
        let rule = page
            .lines()
            .find_map(|line| line.strip_prefix("# `").and_then(|id| id.strip_suffix('`')))
            .expect("literal rule identity");
        if rule.starts_with("type/") != type_aware {
            continue;
        }
        checked += 1;
        let no_finding = page.contains("Current support: `no-sfc-finding`");
        assert_eq!(
            no_finding,
            matches!(
                rule,
                "vapor/require-vapor-attribute"
                    | "vue/no-template-lang"
                    | "vue/no-preprocessor-lang"
                    | "vue/no-script-non-standard-lang"
            ),
            "only the audited unsupported SFC boundaries"
        );
        unavailable += usize::from(no_finding);
        let parser_diagnostic = page.contains("Bad diagnostic: `parser/template`");
        assert_eq!(
            parser_diagnostic,
            rule == "vue/valid-attribute-name",
            "parser-owned invalid attribute spelling"
        );
        parser_owned += usize::from(parser_diagnostic);
        let config = configuration(&page);
        let linter = configure(rule, &config);
        for (heading, should_report) in [("## Bad", true), ("## Good", false)] {
            let (source, language, filename) = example(&page, heading, rule);
            let result = if language == "html" {
                linter.lint_standalone_html(&source, &filename)
            } else if language == "ts" {
                linter.lint_script(&source, &filename)
            } else {
                linter.lint_sfc(&source, &filename)
            };
            let expected_id = if should_report && parser_diagnostic {
                "parser/template"
            } else {
                rule
            };
            let reports = result
                .diagnostics
                .iter()
                .filter(|d| d.rule_name == expected_id)
                .count();
            let should_report = should_report && !no_finding;
            if heading == "## Good"
                && result
                    .diagnostics
                    .iter()
                    .any(|d| d.rule_name.starts_with("parser/"))
            {
                failures.push(cstr!(
                    "{rule} Good must parse successfully: {:?}",
                    result.diagnostics
                ));
            }
            if (reports > 0) != should_report {
                failures.push(cstr!("{rule} {heading} ({filename}): expected report={should_report}, found {reports}; {:?}", result.diagnostics));
            }
        }
    }
    assert_eq!(checked, if type_aware { 6 } else { 252 });
    assert_eq!(unavailable, if type_aware { 0 } else { 4 });
    assert_eq!(parser_owned, if type_aware { 0 } else { 1 });
    eprintln!(
        "documentation corpus: catalog={checked} non_emitted={unavailable} parser_owned={parser_owned} type_aware={type_aware}"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn configuration(page: &str) -> Value {
    parse_configuration(page).expect("canonical authored Vite+ lint configuration")
}

fn parse_configuration(page: &str) -> Result<Value, &'static str> {
    // Notes can contain other configuration examples. Execute only the authored
    // configuration section, including the explicitly unsupported SFC rules.
    let sections: Vec<_> = page
        .split("\n## ")
        .filter_map(|section| {
            section
                .strip_prefix("Configuration (Vite+)\n\n")
                .or_else(|| section.strip_prefix("Configured ID (currently no SFC finding)\n\n"))
        })
        .collect();
    let [section] = sections.as_slice() else {
        return Err("exactly one authored configuration section");
    };
    let block = section
        .strip_prefix("```ts\n")
        .and_then(|text| text.split_once("\n```").map(|(code, _)| code))
        .ok_or("complete TypeScript configuration fence")?;
    let body = block
        .strip_prefix("import { defineConfig } from \"@vizejs/vite-plugin/vite-plus\";\n\n")
        .ok_or("canonical Vite+ defineConfig import")?;
    let text = if let Some(text) =
        body.strip_prefix("export default defineConfig({}).withVue({\n  linter: ")
    {
        text.strip_suffix(",\n});")
    } else {
        body.strip_prefix("export default defineConfig({\n  vue: {\n    linter: ")
            .and_then(|text| text.strip_suffix(",\n  },\n});"))
    }
    .ok_or("canonical Vue linter configuration envelope")?;
    let config: Value =
        serde_json::from_str(text).map_err(|_| "literal JSON linter configuration")?;
    if !config.is_object() {
        return Err("linter configuration object");
    }
    Ok(config)
}

#[test]
fn authored_configuration_preserves_options_and_ignores_note_examples() {
    let config = serde_json::json!({
        "preset": "incremental",
        "rules": { "vue/valid-v-on": "error" },
        "typeAware": true,
        "ruleOptions": {
            "vue/valid-v-on": { "modifiers": ["custom"] },
            "vue/valid-v-slot": { "allowModifiers": true },
            "vue/require-component-registration": { "globals": ["KnownWidget"] },
            "script/no-restricted-members": {
                "members": [{ "object": "window", "property": "alert" }]
            }
        }
    });
    for fluent in [false, true] {
        let page = configuration_page(&config, fluent);
        assert_eq!(configuration(&page), config);
        let with_note = format!("\n## Notes\n\n```ts\n  linter: {{}}\n```\n{page}");
        assert_eq!(configuration(&with_note), config);
        assert_eq!(
            configuration(&page.replace(
                "Configuration (Vite+)",
                "Configured ID (currently no SFC finding)"
            )),
            config
        );
    }
}

#[test]
fn authored_configuration_rejects_missing_ambiguous_and_nonliteral_settings() {
    let page = configuration_page(&serde_json::json!({ "rules": {} }), true);
    for malformed in [
        page.replace("Configuration (Vite+)", "Notes"),
        format!("{page}{page}"),
        page.replace("  linter: ", "  vize: "),
        page.replace("/vite-plus", "/other"),
        page.replace("{\"rules\":{}}", "[]"),
        page.replace("{\"rules\":{}}", "settings"),
        page.replace("{\"rules\":{}}", "{\"rules\":{}} trailing"),
        page.replace("\n});", "\n}"),
        page.replace("\n```\n", "\n"),
    ] {
        assert!(parse_configuration(&malformed).is_err(), "{malformed}");
    }
    let native = configuration_page(&serde_json::json!({ "rules": {} }), false);
    assert!(parse_configuration(&native.replace("  vue: ", "  vize: ")).is_err());
}

#[test]
fn authored_modifier_options_reach_the_real_rule_instances() {
    let rule = "vue/valid-v-on";
    let page = std::fs::read_to_string(reference_root().join("vue-valid-v-on.md"))
        .expect("authored modifier configuration");
    let config = configuration(&page);
    assert_eq!(
        config["ruleOptions"][rule]["modifiers"],
        serde_json::json!(["custom"])
    );
    for (rule, source, options) in [
        (rule, r#"<div @click.custom="handler" />"#, config),
        (
            "vue/valid-v-slot",
            r#"<v-data-table><template #[name].memo="{ item }">{{ item.memo }}</template></v-data-table>"#,
            configuration(&configuration_page(
                &serde_json::json!({ "ruleOptions": { "vue/valid-v-slot": { "allowModifiers": true } } }),
                true,
            )),
        ),
    ] {
        for (settings, expected) in [(&options, false), (&serde_json::json!({}), true)] {
            let result = configure(rule, settings).lint_template(source, "Example.vue");
            assert_eq!(
                result.diagnostics.iter().any(|d| d.rule_name == rule),
                expected,
                "{rule}"
            );
        }
    }
}

fn configuration_page(config: &Value, fluent: bool) -> std::string::String {
    let body = if fluent {
        format!("export default defineConfig({{}}).withVue({{\n  linter: {config},\n}});")
    } else {
        format!("export default defineConfig({{\n  vue: {{\n    linter: {config},\n  }},\n}});")
    };
    format!(
        "\n## Configuration (Vite+)\n\n```ts\nimport {{ defineConfig }} from \"@vizejs/vite-plugin/vite-plus\";\n\n{body}\n```\n"
    )
}

fn configure(rule: &str, config: &Value) -> Linter {
    let mut linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(vec![rule.into()])
        .with_type_aware_lint(config["typeAware"].as_bool().unwrap_or(false));
    let options = &config["ruleOptions"][rule];
    if rule == "vue/valid-v-on"
        && let Some(modifiers) = options["modifiers"].as_array()
    {
        linter = linter.with_valid_v_on_modifiers(
            modifiers
                .iter()
                .map(|name| name.as_str().expect("custom modifier name").into())
                .collect(),
        );
    }
    if rule == "vue/valid-v-slot"
        && let Some(allow) = options["allowModifiers"].as_bool()
    {
        linter = linter.with_valid_v_slot_allow_modifiers(allow);
    }
    if let Some(members) = options["members"].as_array() {
        linter = linter.with_restricted_members(
            members
                .iter()
                .map(|member| {
                    (
                        member["object"].as_str().expect("object").into(),
                        member["property"].as_str().expect("property").into(),
                        member["message"].as_str().map(String::from),
                    )
                })
                .collect(),
        );
    }
    if rule == "vue/require-component-registration"
        && let Some(globals) = options["globals"].as_array()
    {
        linter = linter.with_component_registration_globals(
            globals
                .iter()
                .map(|name| name.as_str().expect("explicit component name").into())
                .collect(),
        );
    }
    if let Some(tokens) = options["tokens"].as_array() {
        linter = linter.with_musea_design_tokens(
            tokens
                .iter()
                .map(|token| {
                    (
                        token["value"].as_str().expect("token value").into(),
                        token["path"].as_str().expect("token path").into(),
                        token["tier"].as_str().unwrap_or("primitive").into(),
                    )
                })
                .collect(),
        );
    }
    linter
}

fn example(page: &str, heading: &str, rule: &str) -> (String, String, String) {
    let section = page.split_once(heading).expect("Bad/Good heading").1;
    let (before, fence) = section.split_once("```").expect("example code fence");
    let (language, code) = fence.split_once('\n').expect("code language");
    let source = code.split_once("\n```").expect("end code fence").0;
    let filename = before
        .lines()
        .find_map(|line| {
            line.strip_prefix('`')
                .and_then(|name| name.strip_suffix('`'))
        })
        .unwrap_or(if rule.starts_with("musea/") {
            "Example.art.vue"
        } else if language == "html" {
            "example.html"
        } else {
            "ExampleCard.vue"
        });
    (source.into(), language.into(), filename.into())
}
