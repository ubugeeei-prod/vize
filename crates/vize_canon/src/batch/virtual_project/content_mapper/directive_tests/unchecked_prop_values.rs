//! #12458: a per-prop check keeps what the props literal leaves unchecked.

use std::path::Path;

use super::super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperSpan, DIRECTIVE_POLICY_IGNORE,
};
use super::super::span_features::CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET;
use super::super::{
    ContentMapperSpanKind, ContentMapperTransformOptions, generate_vue_content_mapper_transform,
    generate_vue_content_mapper_transform_with_options,
};
use super::DIRECTIVE_POLICY_EXPECT;
use crate::virtual_ts::attr_value_declaration;

const APP: &str = "<script setup lang=\"ts\">\nimport Field from './Field.vue'\n</script>\n<template>\n  <Field label=\"x\" :maxlength=\"{}\" />\n</template>\n";
const OWNED: &str = "  const __vize_prop_check_0_maxlength: $AttrValue<__Field_Props_0, \"maxlength\", typeof Field, __Field_0_prop_maxlength> = {} satisfies __Field_0_prop_maxlength;\n";
const PLAIN: &str = "  const __vize_prop_check_0_maxlength: __Field_0_prop_maxlength = {};\n";
const NAME: &str = "__vize_prop_check_0_maxlength";

fn policies(directives: &[ContentMapperDiagnosticDirective], at: usize) -> Vec<usize> {
    directives
        .iter()
        .filter(|directive| directive.0[2] <= at && at < directive.0[3])
        .map(|directive| directive.0[4])
        .collect()
}

/// The start of `statement` and its name's protocol span.
fn check(source: &str, text: &str, statement: &str) -> (usize, ContentMapperSpan) {
    let start = text.find(statement).expect("per-prop check");
    assert_eq!(text.matches(statement).count(), 1);
    let name = start + "  const ".len();
    let attribute = source.find("maxlength").expect("authored attribute name");
    let span = ContentMapperSpan([
        name,
        NAME.len(),
        attribute,
        "maxlength".len(),
        ContentMapperSpanKind::Atom as usize,
        CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET,
    ]);
    (start, span)
}

#[test]
fn echoed_assignment_keeps_only_its_name_visible() {
    let result = generate_vue_content_mapper_transform(Path::new("/project/App.vue"), APP)
        .expect("transform");
    let text = result.text.as_str();
    let (start, span) = check(APP, text, OWNED);
    assert_eq!(
        result
            .mappings
            .iter()
            .filter(|row| row.0[0] == span.0[0])
            .collect::<Vec<_>>(),
        [&span],
        "the name keeps mapping to the attribute name"
    );
    assert_eq!(text.matches("__vize_unchecked_prop_").count(), 0);
    assert_eq!(
        text.matches("_Declared_").count(),
        0,
        "no per-usage input alias"
    );
    assert_eq!(
        text.rsplit_once("\nexport default App;\n")
            .map(|(_, tail)| tail),
        Some(attr_value_declaration(false)),
        "one module helper after the default export"
    );
    let directives = result
        .diagnostic_directives
        .expect("directive table")
        .directives;
    let name_end = span.0[0] + span.0[1];
    let value = start + OWNED.find("{}").expect("value");
    assert_eq!(policies(&directives, span.0[0]), Vec::<usize>::new());
    assert_eq!(policies(&directives, name_end), [DIRECTIVE_POLICY_IGNORE]);
    assert_eq!(policies(&directives, value), [DIRECTIVE_POLICY_IGNORE]);
    let tail = directives
        .iter()
        .find(|directive| directive.0[2] == name_end)
        .expect("tail ignore");
    assert_eq!(tail.0[3], start + OWNED.len(), "{tail:?}");
}

#[test]
fn literal_owners_alone_take_the_owned_form() {
    // A later spread keeps the assignment as the owner (#10069), and so does
    // an ARIA entry; neither literal entry checks the value.
    for (source, statement, aria_plain) in [
        (
            APP.replace("'./Field.vue'", "'./Field.vue'\nconst rest = {}")
                .replace(
                    ":maxlength=\"{}\" />",
                    ":maxlength=\"{}\" v-bind=\"rest\" />",
                ),
            PLAIN,
            0,
        ),
        (
            APP.replace(":maxlength=", ":aria-busy=\"1\" :maxlength="),
            OWNED,
            1,
        ),
    ] {
        let result = generate_vue_content_mapper_transform(Path::new("/project/App.vue"), &source)
            .expect("transform");
        let text = result.text.as_str();
        let (start, span) = check(&source, text, statement);
        let directives = result
            .diagnostic_directives
            .expect("directive table")
            .directives;
        assert_eq!(
            policies(&directives, span.0[0]),
            Vec::<usize>::new(),
            "{source}"
        );
        let value = start + statement.find("{}").expect("value");
        let echoed = if statement == OWNED {
            vec![DIRECTIVE_POLICY_IGNORE]
        } else {
            Vec::new()
        };
        assert_eq!(policies(&directives, value), echoed, "{source}");
        assert_eq!(
            text.matches("  const __vize_prop_check_0_aria_busy: __Field_0_prop_aria_busy = 1;\n")
                .count(),
            aria_plain,
            "ARIA keeps the plain owner"
        );
    }
}

/// `satisfies` must bind to the whole value. A value that ends on its own
/// line would let ASI end the declaration first (TS1434), and a top-level
/// operator weaker than `satisfies` would keep only its right operand.
#[test]
fn split_values_stay_one_owned_expression() {
    for value in ["\n    cond\n      ? 1\n      : {}\n  ", "cond ? 1 : {}"] {
        let source = APP
            .replace(
                ":maxlength=\"{}\"",
                vize_l0::cstr!(":maxlength=\"{value}\"").as_str(),
            )
            .replace("'./Field.vue'", "'./Field.vue'\nconst cond = true");
        let result = generate_vue_content_mapper_transform(Path::new("/project/App.vue"), &source)
            .expect("transform");
        let text = result.text.as_str();
        let opening = "  const __vize_prop_check_0_maxlength: $AttrValue<__Field_Props_0, \"maxlength\", typeof Field, __Field_0_prop_maxlength> = (";
        let start = text.find(opening).expect("owned per-prop check");
        let statement = text
            .get(start..)
            .and_then(|tail| tail.split_once(";\n"))
            .map(|(statement, _)| statement)
            .expect("statement end");
        assert_eq!(
            statement.get(opening.len()..),
            Some(vize_l0::cstr!("{value}) satisfies __Field_0_prop_maxlength").as_str()),
            "{statement}"
        );
    }
}

#[test]
fn authored_expectation_covers_the_visible_name() {
    let source = APP.replace("  <Field", "  <!-- @vue-expect-error -->\n  <Field");
    let result = generate_vue_content_mapper_transform(Path::new("/project/App.vue"), &source)
        .expect("transform");
    let (_, span) = check(&source, result.text.as_str(), OWNED);
    let directives = result
        .diagnostic_directives
        .expect("directive table")
        .directives;
    assert_eq!(policies(&directives, span.0[0]), [DIRECTIVE_POLICY_EXPECT]);
}

/// A fallthrough root checks its literal against `Partial` props, which
/// renders the same wrong-value message as the assignment. Its declared props
/// stay owned by the literal, except for an explicit `undefined` that the
/// relaxed target accepts and the required per-prop contract must reject.
#[test]
fn relaxed_fallthrough_root_keeps_declared_props_with_the_literal() {
    let mut options = ContentMapperTransformOptions::default();
    options.checks.fallthrough_attributes = true;
    options.checks.fallthrough_attributes_specified = true;
    options.checks.check_required_fallthrough_attributes = true;
    let source = "<script setup lang=\"ts\">\nimport Inner from './Inner.vue'\n</script>\n<template>\n  <Inner :label=\"1\" />\n</template>\n";
    let result = generate_vue_content_mapper_transform_with_options(
        Path::new("/project/Typed.vue"),
        source,
        options.clone(),
    )
    .expect("transform");
    let text = result.text.as_str();
    assert_eq!(
        text.matches("  type __Inner_CheckProps_0 = Partial<__Inner_Props_0> & ")
            .count(),
        1,
        "the root usage is relaxed"
    );
    assert_eq!(
        text.matches("  const __vize_prop_check_0_label: $AttrValue<__Inner_Props_0, \"label\", typeof Inner, __Inner_0_prop_label> = 1 satisfies __Inner_0_prop_label;\n")
            .count(),
        1,
        "the declared key, not its `Partial`, decides the owner"
    );

    let undefined = generate_vue_content_mapper_transform_with_options(
        Path::new("/project/Wrapper.vue"),
        &source.replace(":label=\"1\"", ":label=\"undefined\""),
        options,
    )
    .expect("undefined transform");
    assert_eq!(
        undefined
            .text
            .matches("  const __vize_prop_check_0_label: $AttrValue<__Inner_Props_0, \"label\", unknown, __Inner_0_prop_label, {}> = undefined satisfies __Inner_0_prop_label;\n")
            .count(),
        1,
        "the per-prop contract owns explicit undefined at the relaxed root"
    );
}

#[test]
fn component_prop_assignments_are_ignored_only_when_the_literal_owns_them() {
    let source = "<script setup lang=\"ts\">import Child from './Child.vue'; const value = 'wrong'</script>\n<template><Child :count=\"value\" /></template>\n";
    let result = super::transform(source);
    let directives = result
        .diagnostic_directives
        .expect("component prop echo directive");
    let check = result
        .text
        .find("const __vize_prop_check_0_count")
        .expect("per-prop check")
        + "const ".len();
    let tail = result
        .text
        .find(": $AttrValue<__Child_Props_0, \"count\", typeof Child, __Child_0_prop_count> = value satisfies __Child_0_prop_count;")
        .expect("owned per-prop check");
    let literal = result
        .text
        .find("\"count\": value")
        .expect("whole-props entry");

    // Its name keeps only what the literal leaves unchecked (#12458).
    assert!(!super::covered(&directives.directives, check));
    assert!(super::covered(&directives.directives, tail));
    assert!(!super::covered(&directives.directives, literal));
    assert!(directives.unused_expect_directive_diagnostics.is_empty());
}
