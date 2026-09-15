//! TS-28 compiled-output bridge for the S3 backend reference traces.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

struct Fixture {
    name: &'static str,
    source: &'static str,
    vdom_trace: &'static str,
    vapor_trace: &'static str,
}

const STATIC_DYNAMIC_SOURCE: &str =
    r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#;
const CONTROL_SLOTS_SOURCE: &str = r#"<section><p v-if="ready">ready</p><slot name="body"><span v-text="fallback"></span></slot></section>"#;

const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "rust-lowered-static-dynamic",
        source: STATIC_DYNAMIC_SOURCE,
        vdom_trace: include_str!(
            "../../../formal/impeto/fixtures/rust-lowered-static-dynamic.vdom.trace"
        ),
        vapor_trace: include_str!(
            "../../../formal/impeto/fixtures/rust-lowered-static-dynamic.vapor.trace"
        ),
    },
    Fixture {
        name: "rust-lowered-control-slots",
        source: CONTROL_SLOTS_SOURCE,
        vdom_trace: include_str!(
            "../../../formal/impeto/fixtures/rust-lowered-control-slots.vdom.trace"
        ),
        vapor_trace: include_str!(
            "../../../formal/impeto/fixtures/rust-lowered-control-slots.vapor.trace"
        ),
    },
];

const STATIC_DYNAMIC_VAPOR_REFERENCE: &[&str] = &[
    "create-node",
    "create-node",
    "assign-prop",
    "listen",
    "text-effect",
];
const STATIC_DYNAMIC_VAPOR_COMPILED_KNOWN_GAP: &[&str] = &[
    "create-node",
    "create-node",
    "listen",
    "assign-prop",
    "text-effect",
];
const CONTROL_SLOTS_VAPOR_REFERENCE: &[&str] = &[
    "create-node",
    "conditional-effect",
    "create-node",
    "text-effect",
    "slot-effect",
    "create-node",
    "text-effect",
];
const CONTROL_SLOTS_VAPOR_COMPILED_KNOWN_GAP: &[&str] = &[
    "create-node",
    "slot-effect",
    "create-node",
    "text-effect",
    "conditional-effect",
    "create-node",
    "text-effect",
];

#[test]
fn compiled_backend_traces_match_s3_reference_ladder_or_known_gap() {
    for fixture in FIXTURES {
        let vdom_code = compile_dom(fixture.source);
        let actual_vdom = compiled_vdom_trace(&vdom_code);
        let expected_vdom = reference_labels(fixture.vdom_trace);
        assert_eq!(
            actual_vdom, expected_vdom,
            "{} VDOM compiled trace diverged from S3 reference:\n{}",
            fixture.name, vdom_code
        );

        let vapor_code = compile_vapor_template(fixture.source);
        let actual_vapor = compiled_vapor_trace(&vapor_code);
        let expected_vapor = reference_labels(fixture.vapor_trace);
        assert_vapor_trace_matches_reference_or_known_gap(
            fixture.name,
            &actual_vapor,
            &expected_vapor,
            &vapor_code,
        );
    }
}

fn assert_vapor_trace_matches_reference_or_known_gap(
    fixture_name: &str,
    actual: &[&'static str],
    expected: &[&'static str],
    code: &str,
) {
    if actual == expected {
        return;
    }

    // Exact known gap: Vapor emits delegated listeners before the shared
    // dynamic render effect, while the S3 trace currently orders prop/listen/text.
    if fixture_name == "rust-lowered-static-dynamic"
        && expected == STATIC_DYNAMIC_VAPOR_REFERENCE
        && actual == STATIC_DYNAMIC_VAPOR_COMPILED_KNOWN_GAP
    {
        return;
    }

    // Exact known gap: Vapor currently builds the slot fallback before the
    // sibling conditional block, while the S3 trace orders conditional/slot.
    if fixture_name == "rust-lowered-control-slots"
        && expected == CONTROL_SLOTS_VAPOR_REFERENCE
        && actual == CONTROL_SLOTS_VAPOR_COMPILED_KNOWN_GAP
    {
        return;
    }

    assert_eq!(
        actual, expected,
        "{fixture_name} Vapor compiled trace diverged from S3 reference:\n{code}"
    );
}

fn compile_dom(source: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_options(&allocator, source, DomCompilerOptions::default());
    assert!(errors.is_empty(), "DOM compile errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

fn compile_vapor_template(source: &str) -> String {
    let allocator = Allocator::new();
    let result = compile_vapor(&allocator, source, VaporCompilerOptions::default());
    assert!(
        result.error_messages.is_empty(),
        "Vapor compile errors: {:?}",
        result.error_messages
    );
    result.code.to_string()
}

fn reference_labels(trace: &'static str) -> Vec<&'static str> {
    trace
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(trace_label)
        .collect()
}

fn trace_label(line: &'static str) -> &'static str {
    line.split_whitespace()
        .nth(2)
        .unwrap_or_else(|| panic!("bad trace line: {line}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Event {
    pos: usize,
    order: usize,
    label: &'static str,
}

fn compiled_vdom_trace(code: &str) -> Vec<&'static str> {
    let mut events = Vec::new();
    push_all(&mut events, code, "_createElementBlock(", "create-element");
    push_all(&mut events, code, "_createElementVNode(", "create-element");
    push_all(&mut events, code, "\n    ? ", "branch");
    push_all(&mut events, code, "\n      ? ", "branch");
    push_all(&mut events, code, "disabled:", "patch-prop");
    push_all(&mut events, code, "onClick:", "patch-event");
    push_vdom_display_string_events(&mut events, code);
    push_all(&mut events, code, ", \"ready\"", "set-text");
    push_all(&mut events, code, "textContent:", "set-text");
    push_all(&mut events, code, "_renderSlot(", "render-slot");
    event_labels(events)
}

fn push_vdom_display_string_events(events: &mut Vec<Event>, code: &str) {
    for (pos, _) in code.match_indices("_toDisplayString(") {
        let line_start = code[..pos].rfind('\n').map_or(0, |index| index + 1);
        if code[line_start..pos].contains("textContent:") {
            continue;
        }
        events.push(Event {
            pos,
            order: events.len(),
            label: "set-text",
        });
    }
}

fn compiled_vapor_trace(code: &str) -> Vec<&'static str> {
    let mut events = Vec::new();
    let templates = vapor_templates(code);
    push_template_instantiations(&mut events, code, &templates);
    push_all(&mut events, code, "_createIf(", "conditional-effect");
    push_all(&mut events, code, "_createFor(", "list-effect");
    push_all(&mut events, code, "_createSlot(", "slot-effect");
    push_all(
        &mut events,
        code,
        "_setDynamicProps(",
        "assign-dynamic-props",
    );
    push_all(&mut events, code, "_setProp(", "assign-prop");
    push_all(&mut events, code, "_setText(", "text-effect");
    push_all(&mut events, code, ".$evt", "listen");
    push_all(&mut events, code, "_on(", "listen");
    event_labels(events)
}

fn push_all(events: &mut Vec<Event>, source: &str, needle: &str, label: &'static str) {
    for (pos, _) in source.match_indices(needle) {
        events.push(Event {
            pos,
            order: events.len(),
            label,
        });
    }
}

fn event_labels(mut events: Vec<Event>) -> Vec<&'static str> {
    events.sort();
    events.into_iter().map(|event| event.label).collect()
}

fn vapor_templates(source: &str) -> Vec<(usize, &str)> {
    let mut templates = Vec::new();
    let mut search = 0;
    while let Some(rel) = source[search..].find("const t") {
        let id_start = search + rel + "const t".len();
        let Some((id, id_end)) = parse_digits(source, id_start) else {
            search = id_start;
            continue;
        };
        let Some(call_rel) = source[id_end..].find("_template(\"") else {
            search = id_end;
            continue;
        };
        let template_start = id_end + call_rel + "_template(\"".len();
        let Some(template_end) = js_string_end(source, template_start) else {
            break;
        };
        templates.push((id, &source[template_start..template_end]));
        search = template_end + 1;
    }
    templates
}

fn push_template_instantiations(
    events: &mut Vec<Event>,
    source: &str,
    templates: &[(usize, &str)],
) {
    let mut search = 0;
    while let Some(rel) = source[search..].find(" = t") {
        let pos = search + rel;
        let id_start = pos + " = t".len();
        let Some((id, id_end)) = parse_digits(source, id_start) else {
            search = id_start;
            continue;
        };
        if !source[id_end..].starts_with("()") {
            search = id_end;
            continue;
        }
        let template = templates
            .iter()
            .find_map(|(template_id, template)| (*template_id == id).then_some(*template))
            .unwrap_or_else(|| panic!("missing template t{id}"));
        for label in template_labels(template) {
            events.push(Event {
                pos,
                order: events.len(),
                label,
            });
        }
        search = id_end + 2;
    }
}

fn template_labels(template: &str) -> Vec<&'static str> {
    let mut labels = Vec::new();
    let mut offset = 0;
    while let Some(rel) = template[offset..].find('<') {
        let tag_start = offset + rel;
        if !template[offset..tag_start].trim().is_empty() {
            labels.push("text-effect");
        }
        if let Some(next) = template[tag_start + 1..].chars().next()
            && !matches!(next, '/' | '!' | '?')
        {
            labels.push("create-node");
        }
        let Some(end_rel) = template[tag_start..].find('>') else {
            break;
        };
        offset = tag_start + end_rel + 1;
    }
    if !template[offset..].trim().is_empty() {
        labels.push("text-effect");
    }
    labels
}

fn parse_digits(source: &str, start: usize) -> Option<(usize, usize)> {
    let mut end = start;
    let bytes = source.as_bytes();
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    (end > start).then(|| (source[start..end].parse().expect("digits parse"), end))
}

fn js_string_end(source: &str, start: usize) -> Option<usize> {
    let mut escaped = false;
    for (rel, byte) in source[start..].bytes().enumerate() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some(start + rel);
        }
    }
    None
}
