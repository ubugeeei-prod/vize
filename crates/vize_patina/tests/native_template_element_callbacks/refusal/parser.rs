use super::*;

fn parser(message: &str, range: vize_l0::Span, severity: &str) -> serde_json::Value {
    serde_json::json!({
        "rule_name": "parser/template", "severity": severity, "message": message,
        "start": range.start, "end": range.end, "help": null, "labels": [], "fix": null,
    })
}

#[test]
fn late_missing_close_discards_pending_findings_and_preserves_complete_original_parser_order() {
    let source = "<div id='first'/><span title='second'>";
    for locale in LOCALES {
        let log = events();
        let configured = linter(
            vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
            locale,
            HelpLevel::Full,
        );
        assert_eq!(
            configured
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::Header(NativeLintRefusal::Hole)
        );
        assert_eq!(
            *log.lock().unwrap(),
            [
                Event::Root("actual"),
                element_event("actual", "div", 0, None),
                static_event("actual", source, 0, "id", "id='first'", Some("first"))
            ]
        );
        let mut expected = single_expected(
            &configured,
            source,
            "exact.vue",
            &["id='first'", "title='second'"],
        );
        expected["diagnostics"].as_array_mut().unwrap().insert(
            2,
            parser(
                "Element is missing end tag.",
                span(source, "<span title='second'>"),
                "error",
            ),
        );
        expected["error_count"] = 1.into();
        assert_eq!(
            complete(&configured.lint_template(source, "exact.vue")),
            expected
        );
    }
}

#[test]
fn duplicate_header_refuses_before_element_callbacks_and_keeps_original_full_warning_vector() {
    let source = "<div id='first' ID='second'/>";
    for locale in LOCALES {
        let log = events();
        let configured = linter(
            vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
            locale,
            HelpLevel::Full,
        );
        assert_eq!(
            configured
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::Header(NativeLintRefusal::DuplicateAttribute {
                span: span(source, "ID")
            })
        );
        assert_eq!(*log.lock().unwrap(), [Event::Root("actual")]);
        let mut expected = single_expected(
            &configured,
            source,
            "exact.vue",
            &["id='first'", "ID='second'"],
        );
        expected["diagnostics"].as_array_mut().unwrap().insert(
            2,
            parser(
                "Duplicate attribute `ID`. Keeping the repeated attribute so parsing can continue.",
                span(source, "ID"),
                "warning",
            ),
        );
        expected["warning_count"] = 4.into();
        assert_eq!(
            complete(&configured.lint_template(source, "exact.vue")),
            expected
        );
    }
}
