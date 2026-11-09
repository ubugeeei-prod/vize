//! #12458: the companion check owns a value the props literal leaves unchecked.

use std::path::Path;

use super::super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperSpan, DIRECTIVE_POLICY_IGNORE,
};
use super::super::{ContentMapperSpanKind, generate_vue_content_mapper_transform};
use super::DIRECTIVE_POLICY_EXPECT;
use crate::virtual_ts::UNCHECKED_PROP_VALUE_DECLARATION;

const APP: &str = "<script setup lang=\"ts\">\nimport Field from './Field.vue'\n</script>\n<template>\n  <Field label=\"x\" :maxlength=\"{}\" />\n</template>\n";
const DECLARED: &str = "  type __Field_Declared_0 = __Field_Props_0 & $EmitListeners<typeof Field> & $RequiredFallthroughProps<typeof Field> & VueTemplatePublicAttrsType;\n";
const COMPANION: &str = "  const __vize_unchecked_prop_0_maxlength: $UncheckedPropValue<__Field_Declared_0, \"maxlength\", __Field_0_prop_maxlength> = {};\n";
const NAME: &str = "__vize_unchecked_prop_0_maxlength";

fn policies(directives: &[ContentMapperDiagnosticDirective], at: usize) -> Vec<usize> {
    directives
        .iter()
        .filter(|directive| directive.0[2] <= at && at < directive.0[3])
        .map(|directive| directive.0[4])
        .collect()
}

/// The companion statement and its name's protocol span.
fn companion(source: &str, text: &str) -> (usize, ContentMapperSpan) {
    let start = text.find(COMPANION).expect("companion value check");
    assert_eq!(text.matches(COMPANION).count(), 1);
    let name = start + "  const ".len();
    let attribute = source.find("maxlength").expect("authored attribute name");
    let span = ContentMapperSpan([
        name,
        NAME.len(),
        attribute,
        "maxlength".len(),
        ContentMapperSpanKind::Atom as usize,
        0,
    ]);
    (start, span)
}

#[test]
fn echoed_assignment_keeps_its_companion_diagnostic_live() {
    let result = generate_vue_content_mapper_transform(Path::new("/project/App.vue"), APP)
        .expect("transform");
    let text = result.text.as_str();
    let (start, span) = companion(APP, text);
    assert_eq!(
        result
            .mappings
            .iter()
            .filter(|row| row.0[0] == span.0[0])
            .collect::<Vec<_>>(),
        [&span],
        "the name maps to the attribute name without editor features"
    );
    assert_eq!(
        text.rsplit_once("\nexport default App;\n")
            .map(|(_, tail)| tail),
        Some(UNCHECKED_PROP_VALUE_DECLARATION),
        "one module helper after the default export"
    );
    assert_eq!(text.matches("type $UncheckedPropValue<").count(), 1);
    assert_eq!(
        text.matches(DECLARED).count(),
        1,
        "one input alias per usage"
    );
    assert_eq!(
        text.find(DECLARED).map(|at| at + DECLARED.len()),
        text.find("  const __vize_unchecked_prop_0_label:"),
        "the input alias precedes the usage's first companion"
    );
    let directives = result
        .diagnostic_directives
        .expect("directive table")
        .directives;
    let check = text
        .find("__vize_prop_check_0_maxlength:")
        .expect("per-prop check name");
    let value = start + COMPANION.find("= {}").expect("value") + 2;
    assert_eq!(policies(&directives, check), [DIRECTIVE_POLICY_IGNORE]);
    assert_eq!(policies(&directives, span.0[0]), Vec::<usize>::new());
    assert_eq!(policies(&directives, value), [DIRECTIVE_POLICY_IGNORE]);
}

#[test]
fn companion_stays_silent_while_its_assignment_reports() {
    // A later spread keeps the assignment as the owner (#10069), and a
    // derived Pug template carries no echo directives at all.
    for source in [
        APP.replace("'./Field.vue'", "'./Field.vue'\nconst rest = {}")
            .replace(
                ":maxlength=\"{}\" />",
                ":maxlength=\"{}\" v-bind=\"rest\" />",
            ),
        APP.replace(
            "<template>\n  <Field label=\"x\" :maxlength=\"{}\" />\n</template>",
            "<template lang=\"pug\">\nField(label=\"x\" :maxlength=\"{}\")\n</template>",
        ),
    ] {
        let result = generate_vue_content_mapper_transform(Path::new("/project/App.vue"), &source)
            .expect("transform");
        let text = result.text.as_str();
        let (_, span) = companion(&source, text);
        let directives = result
            .diagnostic_directives
            .expect("directive table")
            .directives;
        let check = text
            .find("__vize_prop_check_0_maxlength:")
            .expect("per-prop check name");
        assert_eq!(
            policies(&directives, check),
            Vec::<usize>::new(),
            "{source}"
        );
        assert_eq!(
            policies(&directives, span.0[0]),
            [DIRECTIVE_POLICY_IGNORE],
            "{source}"
        );
    }
}

#[test]
fn authored_expectation_covers_the_live_companion() {
    let source = APP.replace("  <Field", "  <!-- @vue-expect-error -->\n  <Field");
    let result = generate_vue_content_mapper_transform(Path::new("/project/App.vue"), &source)
        .expect("transform");
    let (_, span) = companion(&source, result.text.as_str());
    let directives = result
        .diagnostic_directives
        .expect("directive table")
        .directives;
    assert_eq!(policies(&directives, span.0[0]), [DIRECTIVE_POLICY_EXPECT]);
}

/// The companion value check repeats a static value as an inert copy.
pub(super) fn companion_id_echo(
    source: &str,
    generated: &str,
    id: &str,
) -> ContentMapperDiagnosticDirective {
    let original = source
        .find(vize_carton::cstr!("id=\"{id}\"").as_str())
        .expect("authored id")
        + 4;
    let check = generated
        .find("const __vize_unchecked_prop_0_id:")
        .expect("companion value check");
    let value = vize_carton::cstr!("= \"{id}\";");
    let start = check
        + generated
            .get(check..)
            .expect("companion tail")
            .find(value.as_str())
            .expect("companion value")
        + 2;
    ContentMapperDiagnosticDirective([
        original,
        id.len(),
        start,
        start + id.len() + 2,
        DIRECTIVE_POLICY_IGNORE,
        0,
    ])
}
