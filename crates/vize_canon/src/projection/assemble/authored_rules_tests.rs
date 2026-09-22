use super::{filter, use_vmodel_passive_false_match};
use crate::projection::assemble::{AssembledDiagnostic, AssembledOrigin, AuthoredSource};

const PASSIVE_FALSE_OVERLOAD: &str = "No overload matches this call.\n\
The last overload gave the following error.\n\
Type 'false' is not assignable to type 'true'.";

const CONSUMED_EXPECT_ERROR: &str = r#"useVModel(props, "modelValue", emit, {
  // @ts-expect-error Missing infer for AcceptableValue
  defaultValue: props.defaultValue ?? (multiple.value ? [] : undefined),
  passive: (props.modelValue === undefined) as false,
  deep: true,
});
"#;

#[test]
fn filters_use_vmodel_passive_false_overload_after_consumed_expect_error() {
    let source = CONSUMED_EXPECT_ERROR;
    let mut diagnostics = vec![overload(source, "passive:")];
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(diagnostics, []);
}

#[test]
fn keeps_use_vmodel_passive_false_overload_without_expect_error() {
    let source = r#"useVModel(props, "modelValue", emit, {
  defaultValue: props.defaultValue,
  passive: (props.modelValue === undefined) as false,
  deep: true,
});
"#;
    assert_eq!(
        use_vmodel_passive_false_match(&lines(source), line_of(source, "passive:")),
        None
    );
}

#[test]
fn keeps_use_vmodel_overload_when_expect_error_is_unused() {
    let source = CONSUMED_EXPECT_ERROR;
    let mut diagnostics = vec![
        overload(source, "passive:"),
        unused_directive(source, "@ts-expect-error"),
    ];
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(codes(&diagnostics), [Some(2769), Some(2578)]);
}

#[test]
fn script_files_keep_every_diagnostic() {
    let source = CONSUMED_EXPECT_ERROR;
    let mut diagnostics = vec![overload(source, "passive:")];
    filter(&AuthoredSource::script(source), &mut diagnostics);
    assert_eq!(codes(&diagnostics), [Some(2769)]);
}

#[test]
fn matches_multiline_passive_false_option() {
    let source = r#"useVModel(props, "modelValue", emit, {
  // @ts-expect-error Missing infer for AcceptableValue
  defaultValue: props.defaultValue ?? (multiple.value ? [] : undefined),
  passive:
    (props.modelValue === undefined)
      as false,
  deep: true,
});
"#;
    assert_eq!(
        use_vmodel_passive_false_match(&lines(source), line_of(source, "passive:")),
        Some(line_of(source, "@ts-expect-error"))
    );
}

#[test]
fn matches_use_vmodel_options_after_long_gap() {
    let mut source = vize_carton::String::from("useVModel(props, \"modelValue\", emit, {\n");
    for index in 1..=13 {
        source.push_str(&vize_carton::cstr!("  option{index:02}: true,\n"));
    }
    source.push_str(
        "  // @ts-expect-error Missing infer for AcceptableValue\n  defaultValue: props.defaultValue ?? (multiple.value ? [] : undefined),\n  passive: (props.modelValue === undefined) as false,\n  deep: true,\n});\n",
    );
    assert_eq!(
        use_vmodel_passive_false_match(&lines(&source), line_of(&source, "passive:")),
        Some(line_of(&source, "@ts-expect-error"))
    );
}

#[test]
fn filters_ts_ignore_for_multiline_call_argument_diagnostic() {
    let source = r#"const handler = (event: MouseEvent) => void event
if (handler) {
  // @ts-ignore DOM overload is intentionally wider here
  ;(el as HTMLElement).addEventListener(
    'click',
    handler,
    false
  )
}
"#;
    let mut diagnostics = vec![overload(source, "handler,")];
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(diagnostics, []);
}

#[test]
fn keeps_multiline_call_diagnostic_when_ts_expect_error_is_unused() {
    let source = r#"const handler = (event: MouseEvent) => void event
if (handler) {
  // @ts-expect-error DOM overload is intentionally wider here
  el.addEventListener(
    'click',
    handler,
    false
  )
}
"#;
    let mut diagnostics = vec![
        overload(source, "handler,"),
        unused_directive(source, "@ts-expect-error"),
    ];
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(codes(&diagnostics), [Some(2769), Some(2578)]);
}

#[test]
fn template_diagnostics_are_not_multiline_directive_candidates() {
    let source = "<script setup lang=\"ts\">\n</script>\n<template>\n  <!-- @ts-ignore -->\n  <Comp\n    :a=\"x\"\n  />\n</template>\n";
    let mut diagnostics = vec![diagnostic(source, ":a=", Some(2322), "Type mismatch.")];
    filter(&AuthoredSource::vue(source), &mut diagnostics);
    assert_eq!(codes(&diagnostics), [Some(2322)]);
}

fn lines(source: &str) -> Vec<&str> {
    source.lines().collect()
}

fn codes(diagnostics: &[AssembledDiagnostic<()>]) -> Vec<Option<u32>> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn overload(source: &str, needle: &str) -> AssembledDiagnostic<()> {
    diagnostic(source, needle, Some(2769), PASSIVE_FALSE_OVERLOAD)
}

fn unused_directive(source: &str, needle: &str) -> AssembledDiagnostic<()> {
    diagnostic(
        source,
        needle,
        Some(2578),
        "Unused '@ts-expect-error' directive.",
    )
}

fn diagnostic(
    source: &str,
    needle: &str,
    code: Option<u32>,
    message: &str,
) -> AssembledDiagnostic<()> {
    let start = source.find(needle).unwrap();
    AssembledDiagnostic {
        start,
        end: start + 1,
        code,
        severity: Some(1),
        message: message.into(),
        origin: AssembledOrigin::Checker(()),
    }
}

fn line_of(source: &str, needle: &str) -> usize {
    source
        .lines()
        .position(|line| line.contains(needle))
        .unwrap()
}
