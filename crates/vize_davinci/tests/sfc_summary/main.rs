//! P5-2 — the per-SFC summary.
//!
//! `cargo test -p vize_davinci --test sfc_summary`
//!
//! The summary round-trips in `Full` mode (TS-16). A hot-path optimization
//! inside the component body changes no fingerprint. A signature change
//! invalidates exactly the consumers that recorded that declaration.

#![expect(
    clippy::expect_used,
    clippy::string_slice,
    reason = "tests assert by panicking"
)]

mod fixture;

use fixture::{BARE, BUTTON, button_pages, entry, replace_once, signature, summarize, users};
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_davinci::summary::{AlphaPages, DeclarationId, Facet, SfcSummary, SummaryError, Usage};
use vize_s0::String;

const INLINE_BODY: &str = "const doubled = count.value * 2";
const CACHED_BODY: &str = "const doubled = computed(() => count.value * 2)";

#[test]
fn summary_round_trip_is_exact() {
    let bare = SfcSummary::from_alpha(AlphaPages {
        signature: signature("Bare", ""),
        props: Vec::new(),
        emits: Vec::new(),
        slots: Vec::new(),
        reactivity: Vec::new(),
        components: Vec::new(),
    })
    .expect("bare signature");
    let bare_text = bare.print_to_string(FolioMode::Full);
    assert_eq!(bare_text, BARE);
    assert_eq!(bare.print_to_string(FolioMode::Display), bare_text);
    let parsed = SfcSummary::parse(&bare_text).expect("canonical bare text");
    assert_eq!(parsed, bare);
    assert_eq!(parsed.print_to_string(FolioMode::Full), bare_text);

    // Non-canonical: header order, a blank line, and `params` before `name`.
    let messy = "\
[sfc-summary]
prop-types=1
schema_version=1
component-references=1
reactivity-classes=1
slot-types=1
emit-types=1
component-signature=1

[sfc-summary.component-signature]
params=
name=Bare

";
    let normalized = SfcSummary::parse(messy).expect("messy bare text");
    assert_eq!(normalized, bare);
    assert_eq!(normalized.print_to_string(FolioMode::Full), bare_text);

    let button = summarize(&button_pages());
    let text = button.print_to_string(FolioMode::Full);
    assert_eq!(text, BUTTON);
    assert_eq!(button.len(), 8);
    let parsed = SfcSummary::parse(&text).expect("canonical button text");
    assert_eq!(parsed, button);
    assert_eq!(parsed.print_to_string(FolioMode::Full), text);
    assert_eq!(parsed.print_to_string(FolioMode::Display), text);

    // Unsorted prop lines normalize.
    let swapped = replace_once(
        &text,
        "disabled=boolean\nlabel=string\n",
        "label=string\ndisabled=boolean\n",
    );
    assert_eq!(SfcSummary::parse(&swapped).expect("swapped props"), button);

    assert_eq!(
        SfcSummary::parse("").map(|_| ()),
        Err(FolioError::new(
            0,
            String::from("missing [sfc-summary] header")
        ))
    );
    let schema_line = text
        .lines()
        .position(|line| line == "schema_version=1")
        .expect("schema line")
        + 1;
    let wrong = replace_once(&text, "schema_version=1", "schema_version=2");
    assert_eq!(
        SfcSummary::parse(&wrong).map(|_| ()),
        Err(FolioError::new(
            schema_line,
            String::from("expected `schema_version=1`, found `schema_version=2`")
        ))
    );
    let extra_line = text.lines().count() + 1;
    let mut with_body = text.clone();
    with_body.push_str("[sfc-summary.expression-facts]\n0=body\n");
    assert_eq!(
        SfcSummary::parse(&with_body).map(|_| ()),
        Err(FolioError::new(
            extra_line,
            String::from("unknown section [sfc-summary.expression-facts]")
        ))
    );
}

/// One SFC: its interface α pages, and a body the summary cannot store.
struct Component {
    pages: AlphaPages,
    /// Hot-path shape of the component body. Not a field of [`AlphaPages`].
    body: &'static str,
}

fn button(body: &'static str) -> Component {
    Component {
        pages: button_pages(),
        body,
    }
}

fn summarize_component(component: &Component) -> SfcSummary {
    summarize(&component.pages)
}

#[test]
fn no_ripple_fixture_body_optimization_changes_no_fingerprint() {
    let inline = button(INLINE_BODY);
    let cached = button(CACHED_BODY);
    assert_ne!(inline.body, cached.body);
    assert_eq!(inline.pages, cached.pages);
    let inline = summarize_component(&inline);
    let cached = summarize_component(&cached);
    assert_eq!(inline, cached);
    assert_eq!(
        inline.print_to_string(FolioMode::Full),
        cached.print_to_string(FolioMode::Full)
    );
    assert_eq!(inline.changed(&cached), Vec::<DeclarationId>::new());
    assert_eq!(cached.invalidated(&users(&inline)), Vec::<&str>::new());
    assert_eq!(
        inline.fingerprint(Facet::Prop, "label"),
        cached.fingerprint(Facet::Prop, "label")
    );
}

#[test]
fn a_signature_change_invalidates_exactly_the_recorded_users() {
    let before = summarize(&button_pages());
    let recorded = users(&before);
    let mut pages = button_pages();
    pages.signature.params = String::from("<T>");
    let next = summarize(&pages);

    assert_eq!(
        before.changed(&next),
        vec![DeclarationId::new(Facet::Signature, "Button")]
    );
    assert_eq!(next.invalidated(&recorded), vec!["shell", "header"]);
    assert_eq!(
        before.fingerprint(Facet::Prop, "label"),
        next.fingerprint(Facet::Prop, "label")
    );
    assert_eq!(
        before.fingerprint(Facet::Prop, "disabled"),
        next.fingerprint(Facet::Prop, "disabled")
    );
    assert_eq!(
        before.fingerprint(Facet::Emit, "click"),
        next.fingerprint(Facet::Emit, "click")
    );
    assert_ne!(
        before.fingerprint(Facet::Signature, "Button"),
        next.fingerprint(Facet::Signature, "Button")
    );
}

#[test]
fn a_prop_type_change_invalidates_only_its_recorded_users() {
    let before = summarize(&button_pages());
    let recorded = users(&before);
    let mut pages = button_pages();
    pages.props[0] = entry("label", "string | number");
    let next = summarize(&pages);

    assert_eq!(
        before.changed(&next),
        vec![DeclarationId::new(Facet::Prop, "label")]
    );
    assert_eq!(next.invalidated(&recorded), vec!["card"]);
    assert_eq!(
        before.fingerprint(Facet::Signature, "Button"),
        next.fingerprint(Facet::Signature, "Button")
    );
}

#[test]
fn a_sibling_declaration_does_not_change_any_other_fingerprint() {
    let before = summarize(&button_pages());
    let mut pages = button_pages();
    pages.props.push(entry("size", "\"sm\" | \"md\""));
    // Same contract on two facets must not share a fingerprint.
    pages.props.push(entry("count", "ref"));
    let next = summarize(&pages);

    assert_eq!(
        before.fingerprint(Facet::Prop, "label"),
        next.fingerprint(Facet::Prop, "label")
    );
    assert_eq!(
        before.fingerprint(Facet::Signature, "Button"),
        next.fingerprint(Facet::Signature, "Button")
    );
    assert_eq!(
        before.changed(&next),
        vec![
            DeclarationId::new(Facet::Prop, "count"),
            DeclarationId::new(Facet::Prop, "size"),
        ]
    );
    assert_ne!(
        next.fingerprint(Facet::Prop, "count"),
        next.fingerprint(Facet::Reactivity, "count")
    );
    assert_eq!(
        (
            next.fingerprint(Facet::Prop, "count").is_some(),
            next.fingerprint(Facet::Reactivity, "count").is_some(),
        ),
        (true, true)
    );
}

#[test]
fn a_body_shaped_group_cannot_enter_the_summary() {
    assert_eq!(
        Facet::from_alpha_group("expression-facts", 1),
        Err(SummaryError::NotInterface {
            group: String::from("expression-facts"),
        })
    );
    assert_eq!(
        Facet::from_alpha_group("s3-code-shape", 1),
        Err(SummaryError::NotInterface {
            group: String::from("s3-code-shape"),
        })
    );
    assert_eq!(
        Facet::from_alpha_group("prop-types", 2),
        Err(SummaryError::Schema {
            group: String::from("prop-types"),
            expected: 1,
            found: 2,
        })
    );
    assert_eq!(Facet::from_alpha_group("prop-types", 1), Ok(Facet::Prop));
}

#[test]
fn from_alpha_rejects_a_duplicate_a_bad_name_and_a_body_contract() {
    let mut pages = button_pages();
    pages.props.push(entry("label", "number"));
    assert_eq!(
        SfcSummary::from_alpha(pages).map(|_| ()),
        Err(SummaryError::Duplicate {
            facet: Facet::Prop,
            name: String::from("label"),
        })
    );

    let mut pages = button_pages();
    pages.props.push(entry("bad name", "string"));
    assert_eq!(
        SfcSummary::from_alpha(pages).map(|_| ()),
        Err(SummaryError::BadName {
            facet: Facet::Prop,
            name: String::from("bad name"),
        })
    );

    let mut pages = button_pages();
    pages.emits = vec![entry("click", "Mouse\nEvent")];
    assert_eq!(
        SfcSummary::from_alpha(pages).map(|_| ()),
        Err(SummaryError::BadContract {
            facet: Facet::Emit,
            name: String::from("click"),
        })
    );

    let summary = summarize(&button_pages());
    assert_eq!(
        Usage::record("", &summary, &[]).map(|_| ()),
        Err(SummaryError::EmptyConsumer)
    );
    assert_eq!(
        Usage::record("card", &summary, &[(Facet::Prop, "missing")]).map(|_| ()),
        Err(SummaryError::Unknown {
            facet: Facet::Prop,
            name: String::from("missing"),
        })
    );
    assert_eq!(
        Usage::record(
            "card",
            &summary,
            &[(Facet::Prop, "label"), (Facet::Prop, "label")],
        )
        .map(|_| ()),
        Err(SummaryError::DuplicateUse {
            facet: Facet::Prop,
            name: String::from("label"),
        })
    );
}
