//! The upstream tuple-type span assertion (#3307) is reachable from a public
//! Vize API, not only from the `js_ts_expression` fuzz target.
//!
//! `prefix_identifiers_in_expression` parses without the `typescript` source
//! option, so `<[` is never speculated as type arguments and the class cannot be
//! reached there. `strip_typescript_from_expression` parses `SourceType::ts()`
//! once `needs_typescript_stripping` sees a generic call, so a template
//! expression such as `{{ f<[a?, , b]>(x) }}` aborts any build with debug
//! assertions on (dev / test / the `ci` profile). Release builds are unaffected:
//! the assertion is `debug_assert`-only and the label just gets a garbage span.
//!
//! Both rows below are asserted so the exposure cannot be quietly widened, and
//! so the panic rows fail loudly once the pinned OXC revision reaches 0.141.0.
//! See `upstream_tuple_type_span_assertion.rs` for the full boundary matrix.

use std::panic::{self, AssertUnwindSafe};

use vize_atelier_core::steps::expression::{
    prefix_identifiers_in_expression, strip_typescript_from_expression,
};

/// What a public expression transform does with one input.
#[derive(Clone, Debug, PartialEq, Eq)]
enum TransformOutcome {
    /// Returned the input unchanged — a guard rejection or a failed parse.
    Unchanged,
    /// Rewrote the input to exactly this string.
    Rewritten(String),
    /// Tripped the upstream span assertion with exactly this panic payload.
    UpstreamSpanAssertion(String),
}

fn classify(transform: fn(&str) -> vize_carton::String, source: &str) -> TransformOutcome {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let transformed =
        panic::catch_unwind(AssertUnwindSafe(|| transform(source).as_str().to_string()));
    panic::set_hook(previous_hook);

    match transformed {
        Ok(output) if output == source => TransformOutcome::Unchanged,
        Ok(output) => TransformOutcome::Rewritten(output),
        Err(payload) => TransformOutcome::UpstreamSpanAssertion(
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|text| (*text).to_string())
                })
                .unwrap_or_else(|| "<non-string panic payload>".to_string()),
        ),
    }
}

fn span_assertion() -> TransformOutcome {
    TransformOutcome::UpstreamSpanAssertion("assertion failed: self.start <= self.end".to_string())
}

const SOURCES: [&str; 8] = [
    "<[r?, \u{7}]",
    "f<[a?, \u{7}]>(x)",
    "<[r?, , x]",
    "f<[a?, , b]>(x)",
    "<[r?, ; ]",
    "<[r?, @ ]",
    "<[m?,\n\n",
    "f<[a?, b]>(x)",
];

#[test]
fn identifier_prefixing_never_reaches_the_upstream_span_assertion() {
    let actual: Vec<TransformOutcome> = SOURCES
        .iter()
        .map(|source| classify(prefix_identifiers_in_expression, source))
        .collect();

    assert_eq!(actual, vec![TransformOutcome::Unchanged; SOURCES.len()]);
}

#[test]
fn typescript_stripping_reaches_the_upstream_span_assertion_on_generic_calls() {
    let actual: Vec<(&str, TransformOutcome)> = SOURCES
        .iter()
        .map(|source| (*source, classify(strip_typescript_from_expression, source)))
        .collect();

    assert_eq!(
        actual,
        vec![
            ("<[r?, \u{7}]", TransformOutcome::Unchanged),
            ("f<[a?, \u{7}]>(x)", span_assertion()),
            ("<[r?, , x]", TransformOutcome::Unchanged),
            ("f<[a?, , b]>(x)", span_assertion()),
            ("<[r?, ; ]", TransformOutcome::Unchanged),
            ("<[r?, @ ]", TransformOutcome::Unchanged),
            ("<[m?,\n\n", TransformOutcome::Unchanged),
            ("f<[a?, b]>(x)", TransformOutcome::Unchanged),
        ]
    );
}
