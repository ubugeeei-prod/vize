//! Canon consumes the same AST classifier as Croquis so callback ownership
//! cannot depend on a second handwritten JavaScript scanner.

use vize_croquis::drawer::{EventHandlerExpression, classify_event_handler};

pub(super) fn inline_callback_event_argument(content: &str) -> Option<&'static str> {
    match classify_event_handler(content) {
        EventHandlerExpression::Callback { accepts_event, .. } => {
            Some(if accepts_event { "$event" } else { "" })
        }
        _ => None,
    }
}

pub(super) fn is_callable_handler_reference(content: &str) -> bool {
    matches!(
        classify_event_handler(content),
        EventHandlerExpression::Reference
    )
}

#[cfg(test)]
mod tests {
    use super::{inline_callback_event_argument, is_callable_handler_reference};

    #[test]
    fn inline_callbacks_report_the_event_argument_they_accept() {
        // (handler body, argument the generated call must pass)
        let cases = [
            ("() => f()", Some("")),
            ("(v) => f(v)", Some("$event")),
            ("v => f(v)", Some("$event")),
            ("async (v) => f(v)", Some("$event")),
            ("async v => f(v)", Some("$event")),
            ("((v) => f(v))", Some("$event")),
            (r#"((x = ")") => x)"#, Some("$event")),
            ("(value) /* callback */ => value", Some("$event")),
            ("async /* callback */ (value) => value", Some("$event")),
            ("(fn: (x: string) => string) => fn(\"x\")", Some("$event")),
            ("(fn = (x) => x) => fn(1)", Some("$event")),
            ("function () {}", Some("")),
            ("function (v) { f(v) }", Some("$event")),
            ("function (v) { return () => v }", Some("$event")),
            ("function named(v) { f(v) }", Some("$event")),
            ("async function (v) { f(v) }", Some("$event")),
            ("(async function (v) { f(v) })", Some("$event")),
            ("handler", None),
            ("handlers[key]", None),
            ("jobs.map((job) => job.id)", None),
            ("jobs.map(function (job) { return job.id })", None),
            ("/=>/.test(value)", None),
            // Starts with the `function` keyword's letters but is a call.
            ("functionalHandler(evt)", None),
        ];

        for (content, expected) in cases {
            assert_eq!(
                inline_callback_event_argument(content),
                expected,
                "unexpected inline-callback classification for {content:?}"
            );
        }
    }

    #[test]
    fn undefined_is_not_a_callable_handler_reference() {
        assert!(!is_callable_handler_reference("undefined"));
        assert!(!is_callable_handler_reference("  undefined  "));
    }

    #[test]
    fn actual_handler_references_stay_callable() {
        assert!(is_callable_handler_reference("handler"));
        assert!(is_callable_handler_reference("handlers[key]"));
        assert!(is_callable_handler_reference("form?.submit"));
    }
}
