//! Execute the published examples through the public linter, including context.
use serde_json::Value;
use std::path::{Path, PathBuf};
use vize_l0::{String, cstr};
use vize_patina::{LintPreset, Linter};

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
    assert_eq!(paths.len(), 251, "all current rule implementations");
    let mut checked = 0;
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
            let reports = result
                .diagnostics
                .iter()
                .filter(|d| d.rule_name == rule)
                .count();
            if !should_report
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
    assert_eq!(checked, if type_aware { 6 } else { 245 });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn configuration(page: &str) -> Value {
    let text = page
        .split_once("    vize: ")
        .expect("Vite+ lint configuration")
        .1;
    serde_json::Deserializer::from_str(text)
        .into_iter::<Value>()
        .next()
        .expect("first configuration value")
        .expect("literal configuration object")
}

fn configure(rule: &str, config: &Value) -> Linter {
    let mut linter = Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(vec![rule.into()]))
        .with_type_aware_lint(config["typeAware"].as_bool().unwrap_or(false));
    let options = &config["ruleOptions"][rule];
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
