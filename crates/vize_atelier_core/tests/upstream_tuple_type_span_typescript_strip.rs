//! The upstream tuple-type span assertion (#3307) *was* reachable from a public
//! Vize API, not only from the `js_ts_expression` fuzz target. This file pins
//! that it no longer is.
//!
//! `prefix_identifiers_in_expression` parses without the `typescript` source
//! option, so `<[` is never speculated as type arguments and the class could
//! never be reached there. `strip_typescript_from_expression` parses
//! `SourceType::ts()` once `needs_typescript_stripping` sees a generic call, so
//! a template expression such as `{{ f<[a?, , b]>(x) }}` used to abort any build
//! with debug assertions on (dev / test / the `ci` profile). Release builds were
//! unaffected: the assertion is `debug_assert`-only and the label just got a
//! garbage span.
//!
//! Since the pin bump to oxc `crates_v0.142.0` (#3405) both generic-call rows
//! return `Unchanged` instead of panicking. Every row is still asserted so the
//! exposure cannot be quietly reintroduced by a future pin. See
//! `upstream_tuple_type_span_assertion.rs` for the full boundary matrix.

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

fn classify(transform: fn(&str) -> vize_s0::String, source: &str) -> TransformOutcome {
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
fn typescript_stripping_no_longer_reaches_the_upstream_span_assertion() {
    let actual: Vec<(&str, TransformOutcome)> = SOURCES
        .iter()
        .map(|source| (*source, classify(strip_typescript_from_expression, source)))
        .collect();

    // Every row is `Unchanged`: the two generic calls no longer panic, and a
    // failed parse leaves the expression untouched, so the public transform is
    // a no-op over the whole class.
    assert_eq!(
        actual,
        vec![
            ("<[r?, \u{7}]", TransformOutcome::Unchanged),
            ("f<[a?, \u{7}]>(x)", TransformOutcome::Unchanged),
            ("<[r?, , x]", TransformOutcome::Unchanged),
            ("f<[a?, , b]>(x)", TransformOutcome::Unchanged),
            ("<[r?, ; ]", TransformOutcome::Unchanged),
            ("<[r?, @ ]", TransformOutcome::Unchanged),
            ("<[m?,\n\n", TransformOutcome::Unchanged),
            ("f<[a?, b]>(x)", TransformOutcome::Unchanged),
        ]
    );
}
