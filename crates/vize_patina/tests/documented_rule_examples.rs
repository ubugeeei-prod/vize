//! Execute the published examples through the public linter, including context.
use serde_json::Value;
use std::path::{Path, PathBuf};
use vize_l0::{String, config::ConfigLintRuleOptions, cstr};
use vize_patina::{LintPreset, Linter};

mod documented_catalogue_additions;
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
    let operational_pages = [
        "parser-template.md",
        "parser-sfc.md",
        "parser-jsx.md",
        "type-project-configuration.md",
    ];
    assert_eq!(
        paths.len(),
        328 + operational_pages.len(),
        "all current rule implementations and authored operational diagnostics"
    );
    for page in operational_pages {
        assert!(
            reference_root().join(page).is_file(),
            "operational page: {page}"
        );
    }
    // Parser/project examples have their own full-report and failure controls.
    // Keep the complete registered-rule preset/configuration corpus.
    paths.retain(|path| !operational_pages.contains(&path.file_name().unwrap().to_str().unwrap()));
    assert_eq!(paths.len(), 328, "all current rule implementations");
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
                "vapor/require-vapor-attribute" | "vue/no-script-non-standard-lang"
            ),
            "only the audited unsupported SFC boundaries"
        );
        unavailable += usize::from(no_finding);
        let bad_diagnostic = page.lines().find_map(|line| {
            line.strip_prefix("Bad diagnostic: `")
                .and_then(|id| id.strip_suffix('`'))
        });
        assert_eq!(
            bad_diagnostic,
            match rule {
                "vue/valid-attribute-name" => Some("parser/template"),
                "vue/jsx-uses-vars" => Some("vue/no-unused-components"),
                _ => None,
            },
            "only the documented parser ownership and JSX usage-marking producer"
        );
        parser_owned += usize::from(bad_diagnostic == Some("parser/template"));
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
            let expected_id = bad_diagnostic.unwrap_or(rule);
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
    assert_eq!(checked, if type_aware { 6 } else { 322 });
    assert_eq!(unavailable, if type_aware { 0 } else { 2 });
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
    // configuration section, including explicitly unsupported SFC rules.
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
    let enabled = config["rules"]
        .as_object()
        .map(|rules| {
            rules
                .iter()
                .filter(|(_, severity)| severity.as_str() != Some("off"))
                .map(|(name, _)| String::from(name.as_str()))
                .collect()
        })
        .unwrap_or_else(|| vec![rule.into()]);
    let mut linter = Linter::with_preset(LintPreset::Incremental)
        .with_additional_rules(enabled)
        .with_type_aware_lint(config["typeAware"].as_bool().unwrap_or(false))
        .with_report_unused_disable_directives(
            config["reportUnusedDisableDirectives"]
                .as_bool()
                .unwrap_or(false),
        );
    if let Some(rules) = config["rules"].as_object() {
        linter = linter.with_rule_severity_overrides(
            rules
                .iter()
                .filter_map(|(name, severity)| {
                    let severity = match severity.as_str()? {
                        "error" => vize_patina::Severity::Error,
                        "warn" | "warning" => vize_patina::Severity::Warning,
                        "off" => return None,
                        other => panic!("unsupported authored severity: {other}"),
                    };
                    Some((name.as_str().into(), severity))
                })
                .collect(),
        );
    }
    if !config["ruleOptions"].is_null() {
        let options: ConfigLintRuleOptions = serde_json::from_value(config["ruleOptions"].clone())
            .expect("typed authored rule options");
        linter = linter.with_config_rule_options(&options);
    }
    linter
}

fn example(page: &str, heading: &str, rule: &str) -> (String, String, String) {
    let section = page.split_once(heading).expect("Bad/Good heading").1;
    let (before, fence) = section.split_once("```").expect("example code fence");
    let (info, code) = fence.split_once('\n').expect("code language");
    // The info string can carry display metadata after the language: `vue annotate="add:2"`.
    let language = info.split_whitespace().next().expect("code language");
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
