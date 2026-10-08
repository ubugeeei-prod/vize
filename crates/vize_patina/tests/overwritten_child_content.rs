//! #8285: retained comment children are overwritten by v-html and v-text.

use serde_json::{Value, json};
use std::{fs, path::PathBuf};
use vize_l0::{Allocator, String, cstr};
use vize_patina::rules::{
    ComponentCasing,
    vue::{HyphenationStyle, SfcElementOrderGroup, SfcElementOrderOptions},
};
use vize_patina::{
    HelpLevel, LintPreset, LintResult, Linter, Locale, OutputFormat, Severity, format_results,
};

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/lint/overwritten-child-content-8285")
}

fn read(path: PathBuf) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn controls() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let ids: Vec<String> = serde_json::from_value(read(corpus().join("controls.json"))?)?;
    if ids.len() != 36 {
        return Err("expected36 owned controls".into());
    }
    ids.into_iter()
        .map(|id| read(corpus().join("controls").join(cstr!("{id}.json").as_str())))
        .collect()
}

fn linter() -> Result<Linter, Box<dyn std::error::Error>> {
    let config = read(corpus().join("config.json"))?;
    let rules: Vec<String> = config
        .get("linter")
        .and_then(|linter| linter.get("rules"))
        .and_then(Value::as_object)
        .ok_or("explicit configured rule map")?
        .keys()
        .map(|name| name.as_str().into())
        .collect();
    if rules.len() != 51 {
        return Err("expected51 explicitly configured rules".into());
    }
    Ok(Linter::with_preset(LintPreset::Incremental)
        .with_enabled_rules(Some(rules.clone()))
        .with_rule_severity_overrides(
            rules
                .into_iter()
                .map(|rule| (rule, Severity::Error))
                .collect(),
        )
        .with_attribute_hyphenation(HyphenationStyle::Always)
        .with_component_name_in_template_casing(ComponentCasing::PascalCase)
        .with_sfc_element_order_options(SfcElementOrderOptions {
            order: vec![
                SfcElementOrderGroup::new(vec!["script".into()]),
                SfcElementOrderGroup::new(vec!["template".into()]),
                SfcElementOrderGroup::new(vec!["style".into()]),
            ],
        })
        .with_locale(Locale::En)
        .with_help_level(HelpLevel::None))
}

fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result.diagnostics.iter().map(|d| {
        let labels: Vec<_> = d.labels.iter().map(|label| json!({"message": label.message, "start": label.start, "end": label.end})).collect();
        json!({"rule": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "labels": labels, "fix": d.fix})
    }).collect();
    json!({"filename": result.filename, "errorCount": result.error_count,
        "warningCount": result.warning_count, "diagnostics": diagnostics})
}

#[test]
fn actual_source_api_reports_overwritten_comments_and_preserves_negative_controls() {
    let linter = linter().unwrap();
    let mut wrong = Vec::new();
    for case in controls().unwrap() {
        let id = case["id"].as_str().unwrap();
        let result = linter.lint_sfc(case["source"].as_str().unwrap(), &cstr!("{id}.vue"));
        let count = result
            .diagnostics
            .iter()
            .filter(|d| d.rule_name == "vue/no-child-content")
            .count();
        if count != usize::from(case["expectedChildContent"].as_bool().unwrap()) {
            wrong.push(id.to_owned());
        }
    }
    assert!(wrong.is_empty(), "mismatched whole source cases: {wrong:?}");
}

#[test]
fn comments_exist_in_the_original_parsed_template() {
    let allocator = Allocator::default();
    let (root, errors) =
        vize_armature::Parser::new(&allocator, "<div v-html=\"content\"><!--comment--></div>")
            .parse();
    assert!(errors.is_empty());
    let vize_relief::TemplateChildNode::Element(element) = &root.children[0] else {
        panic!("element");
    };
    assert!(matches!(
        element.children[0],
        vize_relief::TemplateChildNode::Comment(_)
    ));
}

#[test]
fn complete_native_api_and_public_json_reports_repeat_exactly() {
    let linter = linter().unwrap();
    for case in controls().unwrap() {
        let id = case["id"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let expected = read(corpus().join("native").join(cstr!("{id}.json").as_str())).unwrap();
        for _ in 0..2 {
            let filename = cstr!("{id}.vue");
            let result = linter.lint_sfc(source, &filename);
            assert_eq!(complete(&result), expected["api"], "{id}");
            let report: Value = serde_json::from_str(&format_results(
                &[result],
                &[(filename, source.into())],
                OutputFormat::Json,
            ))
            .unwrap();
            assert_eq!(report, expected["packet"], "{id}");
        }
    }
}
