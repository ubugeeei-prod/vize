//! Handler scenarios shared by compiler and independent browser observations.

/// Valid authored programs with deliberately wrong delivery or scope ownership.
/// Compile these through the same entry point; mutations never depend on how
/// generated JavaScript is printed.
pub fn mutant_handler(name: &str) -> Option<&'static str> {
    Some(match name {
        "event-reference" => "event => void event",
        "inline" => "() => save($event.type, $event.target.id)",
        "capture" => "$event => save($event.type, $event.target.id)",
        "forward-const" => {
            "event => { const deliver = () => save($event.type, $event.target.id ?? 'missing'); deliver() }"
        }
        "loop-for" => {
            "() => { for (var $event = 0; $event < 1; $event++) {} save($event.type, $event.target) }"
        }
        _ => return None,
    })
}

pub const CASES: &[(&str, &str, &str, &str, bool)] = &[
    (
        "temporal-dead-zone",
        "event => { const $event = $event; save(event.type, event.target.id) }",
        "value",
        "error:ReferenceError",
        false,
    ),
    (
        "forward-const",
        "event => { const deliver = () => save($event.type, $event.target.id ?? 'missing'); const $event = event; deliver() }",
        "value",
        "event",
        false,
    ),
    (
        "forward-function",
        "event => { const deliver = () => { if (typeof $event === 'function') $event(event) }; function $event(e) { save(e.type, e.target.id) } deliver() }",
        "value",
        "event",
        false,
    ),
    (
        "forward-class",
        "event => { const deliver = () => { if (typeof $event === 'function') new $event().run() }; class $event { run() { save(event.type, event.target.id) } } deliver() }",
        "value",
        "event",
        false,
    ),
    (
        "loop-for",
        "() => { for (let $event = 0; $event < 1; $event++) {} save($event.type, $event.target) }",
        "value",
        "binding",
        false,
    ),
    (
        "loop-in",
        "() => { for (const $event in { a: 1 }) {} save($event.type, $event.target) }",
        "value",
        "binding",
        false,
    ),
    (
        "loop-of",
        "() => { for (const $event of [1]) {} save($event.type, $event.target) }",
        "value",
        "binding",
        false,
    ),
    (
        "inline-program-var",
        "const deliver = () => { if (value) save(value.type, value.target.id) }; { var value = $event } deliver()",
        "value",
        "event",
        false,
    ),
    (
        "switch-scope",
        "() => { switch ($event.type) { case 'setup-a': case 'setup-b': const $event = 42; break } save($event.type, $event.target) }",
        "value",
        "binding",
        false,
    ),
    (
        "class-self",
        "event => { const Handler = class $event { run() { if (typeof $event === 'function') save(event.type, event.target.id) } }; new Handler().run() }",
        "value",
        "event",
        false,
    ),
    (
        "class-static-var",
        "() => { class Dummy { static { var $event = 42 } } save($event.type, $event.target) }",
        "value",
        "binding",
        false,
    ),
    ("method", "save", "value", "event", false),
    (
        "var-function-scope",
        "event => { const deliver = () => save($event.type, $event.target.id ?? 'missing'); { var $event = event } deliver() }",
        "value",
        "event",
        false,
    ),
    (
        "nested-function-var",
        "event => { (() => { var $event = event })(); save($event.type, $event.target) }",
        "value",
        "binding",
        false,
    ),
    (
        "rest-arrow",
        "(...$event) => save($event[0].type, $event[0].target.id)",
        "value",
        "event",
        false,
    ),
    (
        "rest-function",
        "function (...$event) { save($event[0].type, $event[0].target.id) }",
        "value",
        "event",
        false,
    ),
    (
        "destructure",
        "({ type: $event, target }) => save($event, target.id)",
        "value",
        "event",
        false,
    ),
    (
        "named-function",
        "function $event(event) { if (typeof $event === 'function') save(event.type, event.target.id) }",
        "value",
        "event",
        false,
    ),
    (
        "block-capture",
        "() => { { const $event = 42; } save($event.type, $event.target) }",
        "value",
        "binding",
        false,
    ),
    (
        "catch-capture",
        "() => { try { throw 42 } catch ($event) {} save($event.type, $event.target) }",
        "value",
        "binding",
        false,
    ),
    ("member", "actions.save", "value", "event", false),
    ("parenthesized", "(actions.save)", "value", "event", false),
    (
        "reference-statement",
        "save; count++",
        "value",
        "none",
        true,
    ),
    ("event-reference", "$event", "function", "event", false),
    ("event-member", "$event.target", "member", "event", false),
    (
        "event-computed",
        "$event['target']",
        "member",
        "event",
        false,
    ),
    ("event-optional", "$event?.target", "member", "event", false),
    (
        "inline",
        "save($event.type, $event.target.id)",
        "value",
        "event",
        false,
    ),
    (
        "statements",
        "count++; save($event.type, $event.target.id)",
        "value",
        "event",
        true,
    ),
    (
        "arrow",
        "event => save(event.type, event.target.id)",
        "value",
        "event",
        false,
    ),
    (
        "shadow",
        "$event => save($event.type, $event.target.id)",
        "value",
        "event",
        false,
    ),
    (
        "function",
        "function ($event) { save($event.type, $event.target.id) }",
        "value",
        "event",
        false,
    ),
    (
        "capture",
        "() => save($event.type, $event.target)",
        "value",
        "binding",
        false,
    ),
    (
        "shorthand",
        "() => save(({ $event }).$event.type, $event.target)",
        "value",
        "binding",
        false,
    ),
    (
        "nested-inline",
        "count++; (() => save($event.type, $event.target.id))()",
        "value",
        "event",
        true,
    ),
    ("missing-reference", "$event", "missing", "none", false),
    ("null-reference", "$event", "null", "none", false),
    ("value-reference", "$event", "number", "none", false),
];
