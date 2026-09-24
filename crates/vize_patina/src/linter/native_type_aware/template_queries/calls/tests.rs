use super::{FloatingPromiseRange, OxcAllocator, RelativeRange, SourceType, TemplateCallRanges};

fn collect_template_call_ranges(
    source: &str,
    allow_statement_fallback: bool,
    include_callees: bool,
    include_floating_promises: bool,
) -> TemplateCallRanges {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("template.ts").unwrap_or_default();
    super::collect_template_call_ranges(
        &allocator,
        source_type,
        source,
        allow_statement_fallback,
        include_callees,
        include_floating_promises,
    )
}

fn range_slices<'a>(source: &'a str, ranges: &[RelativeRange]) -> Vec<&'a str> {
    ranges
        .iter()
        .map(|range| &source[range.start as usize..range.end as usize])
        .collect()
}

fn promise_slices<'a>(source: &'a str, ranges: &[FloatingPromiseRange]) -> Vec<&'a str> {
    ranges
        .iter()
        .map(|range| &source[range.start as usize..range.end as usize])
        .collect()
}

#[test]
fn collects_callees_and_floating_promises_from_one_expression_parse() {
    let source = "enabled && save()";
    let ranges = collect_template_call_ranges(source, false, true, true);

    assert_eq!(range_slices(source, &ranges.callees), vec!["save"]);
    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["save()"]
    );
}

#[test]
fn collects_statement_fallback_callees_for_event_handlers() {
    let source = "if (enabled) { save(); track() }";
    let ranges = collect_template_call_ranges(source, true, true, false);

    assert_eq!(range_slices(source, &ranges.callees), vec!["save", "track"]);
    assert!(ranges.floating_promises.is_empty());
}

#[test]
fn collects_statement_fallback_callees_after_expression_prefix() {
    let source = "safe(); anyHandler()";
    let ranges = collect_template_call_ranges(source, true, true, false);

    assert_eq!(
        range_slices(source, &ranges.callees),
        vec!["safe", "anyHandler"]
    );
    assert!(ranges.floating_promises.is_empty());
}

#[test]
fn collects_statement_fallback_floating_promises() {
    let source = "save(); track()";
    let ranges = collect_template_call_ranges(source, true, false, true);

    assert!(ranges.callees.is_empty());
    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["save()", "track()"]
    );
}

#[test]
fn collects_statement_fallback_floating_promises_inside_control_flow() {
    let source = "if (enabled) { save(); track() }";
    let ranges = collect_template_call_ranges(source, true, false, true);

    assert!(ranges.callees.is_empty());
    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["save()", "track()"]
    );
}

#[test]
fn collects_bare_event_handler_references_as_floating_candidates() {
    let source = "save";
    let ranges = collect_template_call_ranges(source, true, false, true);

    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["save"]
    );
}

#[test]
fn collects_member_event_handler_references_as_floating_candidates() {
    let source = "actions.save";
    let ranges = collect_template_call_ranges(source, true, false, true);

    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["actions.save"]
    );
}

#[test]
fn collects_optional_member_event_handler_references_as_floating_candidates() {
    let source = "actions?.save";
    let ranges = collect_template_call_ranges(source, true, false, true);

    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["actions?.save"]
    );
}

#[test]
fn collects_computed_member_event_handler_references_as_floating_candidates() {
    let source = "actions[method]";
    let ranges = collect_template_call_ranges(source, true, false, true);

    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["actions[method]"]
    );
}

#[test]
fn collects_optional_computed_member_event_handler_references_as_floating_candidates() {
    let source = "actions?.[method]";
    let ranges = collect_template_call_ranges(source, true, false, true);

    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["actions?.[method]"]
    );
}

#[test]
fn computed_member_expressions_prefer_binding_probes() {
    let computed = collect_template_call_ranges("payload[key]", false, true, false);
    let optional = collect_template_call_ranges("payload?.[key]", false, true, false);
    let call = collect_template_call_ranges("payload[key]()", false, true, false);
    let static_member = collect_template_call_ranges("payload.key", false, true, false);

    assert!(computed.probe_expression_binding);
    assert!(optional.probe_expression_binding);
    assert!(!call.probe_expression_binding);
    assert!(!static_member.probe_expression_binding);
}

#[test]
fn ignores_bare_references_without_event_fallback() {
    let source = "save";
    let ranges = collect_template_call_ranges(source, false, false, true);

    assert!(ranges.floating_promises.is_empty());
}

#[test]
fn reports_then_without_rejection_handler_as_floating() {
    let source = "save().then(() => {})";
    let ranges = collect_template_call_ranges(source, false, false, true);

    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["save().then(() => {})"]
    );
}

#[test]
fn ignores_then_with_rejection_handler() {
    let source = "save().then(() => {}, report)";
    let ranges = collect_template_call_ranges(source, false, false, true);

    assert!(ranges.floating_promises.is_empty());
}

#[test]
fn reports_empty_catch_as_floating() {
    let source = "save().catch()";
    let ranges = collect_template_call_ranges(source, false, false, true);

    assert_eq!(
        promise_slices(source, &ranges.floating_promises),
        vec!["save().catch()"]
    );
}
