//! The deferred path must retain diagnostics after its parse arenas are gone.

use super::super::{RULE_NO_UNSAFE_TEMPLATE_BINDING, RULE_REQUIRE_TYPED_PROPS, TypeAwareFailure};
use super::{PreparedSfc, prepare_with_descriptor};
use crate::{LintPreset, Linter};
use vize_l0::String;

fn configured(rules: &[&str]) -> Linter {
    Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(true)
        .with_enabled_rules(Some(rules.iter().map(|rule| (*rule).into()).collect()))
}

fn prepare(linter: &Linter, source: &str) -> PreparedSfc {
    let descriptor = crate::linter::script_rules::parse_sfc_for_lint(source, "Owned.vue").unwrap();
    prepare_with_descriptor(linter, source, "Owned.vue", &descriptor, false)
}

fn assert_owned<T: Send + 'static>(_: &T) {}

#[test]
fn prepared_sfc_outlives_source_descriptor_and_template_arena() {
    let linter = configured(&[RULE_NO_UNSAFE_TEMPLATE_BINDING, "css/no-important"]);
    let prepared = {
        let source = String::from(
            "<script setup lang=\"ts\">const value: any = 1;</script>\n\
             <template><p>{{ value }}</p></template>\n\
             <style>p { color: red !important }</style>",
        );
        prepare(&linter, &source)
    };
    assert_owned(&prepared);
    assert!(prepared.virtual_content().is_some());
    let result = prepared.finish(
        &linter,
        Some(TypeAwareFailure::Runtime("owned checker failure".into())),
    );
    let rules = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect::<Vec<_>>();
    assert_eq!(rules, ["type/corsa-runtime", "css/no-important"]);
    assert_eq!(result.warning_count, 2);
    assert_eq!(result.error_count, 0);
}

#[test]
fn selected_runtime_query_failure_is_an_error_and_preserves_css_findings() {
    let linter = configured(&[RULE_NO_UNSAFE_TEMPLATE_BINDING, "css/no-important"]);
    let source = "<script setup lang=\"ts\">const value: any = 1;</script>\n\
                  <template><p>{{ value }}</p></template>\n\
                  <style>p { color: red !important }</style>";
    let unchanged = prepare(&linter, source).finish(&linter, None);
    let result = prepare(&linter, source).finish(
        &linter,
        Some(TypeAwareFailure::Query(
            "Native TypeScript runtime /project/node_modules/@tsrs/linux-x64-gnu/tsrs failed: protocol error: Invalid boolean checker AST: unsupported protocol version".into(),
        )),
    );
    assert_eq!((result.error_count, result.warning_count), (1, 1));
    assert_eq!(result.diagnostics.len(), unchanged.diagnostics.len() + 1);
    let query = &result.diagnostics[0];
    assert_eq!(
        (
            query.rule_name,
            query.severity,
            query.message.as_str(),
            query.start,
            query.end,
            query.help.as_deref(),
            query.labels.len(),
            query.fix.is_none(),
        ),
        (
            "type/corsa-runtime",
            crate::Severity::Error,
            "Native TypeScript runtime /project/node_modules/@tsrs/linux-x64-gnu/tsrs failed: protocol error: Invalid boolean checker AST: unsupported protocol version",
            0,
            0,
            Some(
                "Type-aware lint could not complete with the selected native runtime. Use a compatible runtime or configure `typeChecker.corsaPath`, `CORSA_PATH`, or `TSGO_PATH` to its executable."
            ),
            0,
            true,
        )
    );
    assert_eq!(
        format!("{:?}", &result.diagnostics[1..]),
        format!("{:?}", unchanged.diagnostics)
    );
}

#[test]
fn no_query_sfc_keeps_static_type_findings_before_css_and_ignores_checker_failure() {
    let linter = configured(&[RULE_REQUIRE_TYPED_PROPS, "css/no-important"]);
    let prepared = prepare(
        &linter,
        "<script setup>defineProps(['title'])</script>\n\
         <template><p /></template>\n\
         <style>p { color: red !important }</style>",
    );
    assert!(prepared.virtual_content().is_none());
    let result = prepared.finish(
        &linter,
        Some(TypeAwareFailure::Runtime("unused project".into())),
    );
    let rules = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect::<Vec<_>>();
    assert_eq!(rules, [RULE_REQUIRE_TYPED_PROPS, "css/no-important"]);
    assert_eq!(result.warning_count, 2);
    assert_eq!(result.error_count, 0);
}

#[test]
fn query_plan_keeps_project_configuration_error_before_css() {
    let linter = configured(&[RULE_NO_UNSAFE_TEMPLATE_BINDING, "css/no-important"]);
    let prepared = prepare(
        &linter,
        "<script setup lang=\"ts\">const value: any = 1;</script>\n\
         <template><p>{{ value }}</p></template>\n\
         <style>p { color: red !important }</style>",
    );
    assert!(prepared.virtual_content().is_some());
    let result = prepared.finish(
        &linter,
        Some(TypeAwareFailure::ProjectConfiguration {
            message: "invalid config".into(),
            report: true,
        }),
    );
    let rules = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect::<Vec<_>>();
    assert_eq!(rules, ["type/project-configuration", "css/no-important"]);
    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 1);
}
