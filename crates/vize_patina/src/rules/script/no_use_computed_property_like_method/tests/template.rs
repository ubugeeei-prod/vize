//! Template half (#3414 A): a computed called like a method from the template.
//!
//! Because this half *creates* findings from template evidence, the over-match
//! probes are as load-bearing as the positive cases: each asserts the full
//! finding set, and the negative ones assert it is exactly empty.

use super::{findings, lint_sfc, none};
use crate::diagnostic::Severity;

const OPTIONS: &str = r#"<script>
export default {
  computed: {
    total() { return 1; },
  },
};
</script>
"#;

/// The finding the call written as `call` produces. `call` is located inside
/// the `<template>` block, since the shared script declares `total()` too.
fn called(sfc: &str, call: &str, name: &str) -> (&'static str, Severity, u32, u32, &'static str) {
    let template = sfc.find("<template>").expect("template block");
    let start = template + sfc[template..].find(call).expect("call expression");
    (
        "script/no-use-computed-property-like-method",
        Severity::Error,
        start as u32,
        (start + call.len()) as u32,
        match name {
            "total" => "'total' is a computed property and must not be called like a method.",
            other => panic!("unexpected computed name {other}"),
        },
    )
}

/// An SFC made of the shared Options API script plus `template`.
fn sfc(template: &str) -> std::string::String {
    format!("{OPTIONS}\n<template>\n{template}\n</template>\n")
}

// --- The recovered case: a computed called from a template expression ------

#[test]
fn reports_a_computed_called_in_an_interpolation_issue_3414() {
    // Exact reproduction from #3414 A.
    let source = sfc("  <div>{{ total() }}</div>");
    assert_eq!(
        findings(&lint_sfc(&source)),
        vec![called(&source, "total()", "total")]
    );
}

#[test]
fn reports_a_computed_called_in_a_bound_attribute() {
    let source = sfc(r#"  <div :title="total()"></div>"#);
    assert_eq!(
        findings(&lint_sfc(&source)),
        vec![called(&source, "total()", "total")]
    );
}

#[test]
fn reports_a_computed_called_in_an_inline_handler() {
    let source = sfc(r#"  <button @click="total()">go</button>"#);
    assert_eq!(
        findings(&lint_sfc(&source)),
        vec![called(&source, "total()", "total")]
    );
}

#[test]
fn reports_a_computed_called_inside_a_v_for_body() {
    let source = sfc(r#"  <ul><li v-for="row in rows" :key="row">{{ total() }}</li></ul>"#);
    assert_eq!(
        findings(&lint_sfc(&source)),
        vec![called(&source, "total()", "total")]
    );
}

#[test]
fn reports_a_computed_called_as_a_nested_argument() {
    let source = sfc(r#"  <div>{{ String(total()) }}</div>"#);
    assert_eq!(
        findings(&lint_sfc(&source)),
        vec![called(&source, "total()", "total")]
    );
}

// --- The computed is read, not called: exactly zero findings ---------------

#[test]
fn ignores_a_computed_read_as_a_value() {
    let source = sfc("  <div>{{ total }}</div>");
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn ignores_a_method_called_from_the_template() {
    let source = r#"<script>
export default {
  computed: {
    total() { return 1; },
  },
  methods: {
    reload() { return 2; },
  },
};
</script>

<template>
  <button @click="reload()">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(source)), none());
}

// --- Over-match probes: none of these may manufacture a finding ------------

#[test]
fn ignores_a_call_inside_an_html_comment() {
    let source = sfc("  <div><!-- total() --></div>");
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn ignores_a_call_in_a_text_node_or_plain_attribute() {
    let source = sfc(r#"  <p title="total()">total()</p>"#);
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn ignores_a_call_inside_a_string_literal() {
    let source = sfc(r#"  <button @click="console.log('total()')">go</button>"#);
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn ignores_a_call_inside_a_v_pre_region() {
    let source = sfc("  <pre v-pre>{{ total() }}</pre>");
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn ignores_an_identifier_that_merely_ends_with_the_computed_name() {
    let source = sfc("  <div>{{ subtotal() }}</div>");
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn ignores_a_member_call_whose_property_is_the_computed_name() {
    // `bus.total()` dispatches on another object, not on this component.
    let source = sfc("  <div>{{ bus.total() }}</div>");
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn ignores_a_slot_variable_that_shadows_the_computed_name() {
    let source = sfc(r#"  <Child v-slot="{ total }">{{ total() }}</Child>"#);
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn ignores_a_v_for_alias_that_shadows_the_computed_name() {
    let source = sfc(r#"  <ul><li v-for="total in rows" :key="total">{{ total() }}</li></ul>"#);
    assert_eq!(findings(&lint_sfc(&source)), none());
}

#[test]
fn still_reports_a_computed_after_a_shadowing_subtree_ends() {
    let source = sfc(
        "  <ul><li v-for=\"total in rows\" :key=\"total\">{{ total() }}</li></ul>\n  <p>{{ total() }}</p>",
    );
    let last = source.rfind("total()").expect("second call");
    assert_eq!(
        findings(&lint_sfc(&source)),
        vec![(
            "script/no-use-computed-property-like-method",
            Severity::Error,
            last as u32,
            (last + "total()".len()) as u32,
            "'total' is a computed property and must not be called like a method.",
        )]
    );
}

#[test]
fn ignores_a_template_when_the_script_declares_no_computed() {
    let source = r#"<script>
export default {
  methods: {
    total() { return 1; },
  },
};
</script>

<template>
  <div>{{ total() }}</div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(source)), none());
}

// --- The pre-existing script-only subset must keep working -----------------

#[test]
fn still_reports_a_this_member_call_through_the_sfc_path() {
    let source = r#"<script>
export default {
  computed: {
    total() { return 1; },
  },
  methods: {
    go() { return this.total(); },
  },
};
</script>

<template>
  <div>{{ total }}</div>
</template>
"#;
    let call = source.find("this.total()").expect("member call");
    assert_eq!(
        findings(&lint_sfc(source)),
        vec![(
            "script/no-use-computed-property-like-method",
            Severity::Error,
            call as u32,
            (call + "this.total()".len()) as u32,
            "'total' is a computed property and must not be called like a method.",
        )]
    );
}
