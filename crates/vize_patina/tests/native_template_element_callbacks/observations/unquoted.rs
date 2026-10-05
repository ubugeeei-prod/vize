//! Preserve the exact rejected positive input as a complete original control.

use super::super::support::*;
use vize_patina::{
    Linter,
    native::{NativeLintRefusal, template::NativeTemplateLintRefusal as Refusal},
};

pub(super) fn refused_original(configured: &Linter, source: &str, log: &Events) {
    assert_eq!(
        configured
            .lint_native_template(source, "/元/exact.vue")
            .unwrap_err(),
        Refusal::Header(NativeLintRefusal::Hole)
    );
    assert_eq!(
        *log.lock().unwrap(),
        [
            Event::Root("actual"),
            element_event("actual", "div", 1, None),
            static_event("actual", source, 0, "disabled", "disabled", None),
            static_event(
                "actual",
                source,
                1,
                "title",
                "title = \"&amp; 界\"",
                Some("&amp; 界")
            ),
            static_event(
                "actual",
                source,
                2,
                "data-id",
                "data-id='{{ opaque }}'",
                Some("{{ opaque }}")
            )
        ]
    );
    let mut expected = single_expected(
        configured,
        source,
        "/元/exact.vue",
        &[
            "disabled",
            "title = \"&amp; 界\"",
            "data-id='{{ opaque }}'",
            "lang=en/",
        ],
    );
    let opening = span(source, "<span lang=en/>");
    expected["diagnostics"].as_array_mut().unwrap().insert(
        4,
        serde_json::json!({
            "rule_name": "parser/template", "severity": "error",
            "message": "Element is missing end tag.", "start": opening.start, "end": opening.end,
            "help": null, "labels": [], "fix": null,
        }),
    );
    expected["error_count"] = 1.into();
    assert_eq!(
        complete(&configured.lint_template(source, "/元/exact.vue")),
        expected
    );
}
