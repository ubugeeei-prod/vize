//! TS-16 for `#[derive(Folio)]` pages (P2-4).
//!
//! Per derived type: `print(parse(t)) == t` byte-exact in `Full` mode for
//! canonical text, `parse(print(v)) == v` structurally for values, and
//! `Display` explicitly carrying **no** round-trip law - for a derived page
//! `Display` prints the same canonical text as `Full`, because elision is a
//! semantic decision and the derive makes none.
//!
//! Exercised on the real derived type (`BudgetObserver`) and on a local
//! type covering every supported field kind (bool / integer / `String`
//! scalars, a `Vec` list section, an `FxHashMap` sorted map section).

use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_davinci::pass::BudgetObserver;
use vize_s0::{FxHashMap, String, cstr};

/// Every supported field kind on one page.
#[derive(Debug, Default, PartialEq, Folio)]
struct SamplePage {
    title: String,
    enabled: bool,
    count: u32,
    notes: Vec<String>,
    weights: FxHashMap<String, u32>,
}

/// Canonical text: scalars in declaration order inside the header, then the
/// sections in declaration order, map entries sorted by key, one blank line
/// after every section, LF.
const CANONICAL: &str = "\
[sample-page]
title=hello world
enabled=true
count=7

[sample-page.notes]
first note
second note

[sample-page.weights]
alpha=1
beta=2

";

const BUDGET_CANONICAL: &str = "\
[budget-observer]
walks=2
passes=3
analyses=0
pipelines=1
failures=0

";

fn sample() -> SamplePage {
    SamplePage::parse(CANONICAL).expect("canonical text parses")
}

#[test]
fn full_print_is_identity_on_canonical_text() {
    assert_eq!(
        sample().print_to_string(FolioMode::Full).as_str(),
        CANONICAL
    );
}

#[test]
fn parse_print_is_structural_identity() {
    let value = sample();
    let printed = value.print_to_string(FolioMode::Full);
    let reparsed = SamplePage::parse(printed.as_str()).expect("printed text parses");
    assert_eq!(reparsed, value);
}

#[test]
fn a_hand_built_value_round_trips_structurally() {
    let mut weights = FxHashMap::default();
    // Inserted in reverse key order: the printer sorts (rule 1), a hash map
    // has no order to preserve.
    weights.insert(String::from("beta"), 2u32);
    weights.insert(String::from("alpha"), 1u32);
    let value = SamplePage {
        title: String::from("hello world"),
        enabled: true,
        count: 7,
        notes: [String::from("first note"), String::from("second note")]
            .into_iter()
            .collect(),
        weights,
    };
    assert_eq!(value.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    let reparsed = SamplePage::parse(CANONICAL).expect("canonical text parses");
    assert_eq!(reparsed, value);
}

#[test]
fn non_canonical_input_is_normalized_by_the_first_print() {
    // Scalar lines and sections out of order, extra blank lines, an
    // unsorted map, a non-canonical integer spelling: parse accepts all of
    // it (leniency exactly where print normalizes) and the first print is
    // canonical.
    let scrambled = "\
[sample-page]
count=007

enabled=true
title=hello world


[sample-page.weights]
beta=2

alpha=1

[sample-page.notes]
first note

second note
";
    let folio = SamplePage::parse(scrambled).expect("non-canonical text parses");
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), CANONICAL);
    assert_eq!(folio, sample());
}

#[test]
fn empty_sections_are_omitted() {
    let text = "\
[sample-page]
title=
enabled=false
count=0

";
    let folio = SamplePage::parse(text).expect("header-only text parses");
    assert_eq!(folio, SamplePage::default());
    assert_eq!(folio.print_to_string(FolioMode::Full).as_str(), text);
}

#[test]
fn display_mode_prints_the_full_text_and_carries_no_law() {
    // A derived page has no Display elision: eliding is a semantic decision
    // and the derive is the mechanical trio only. The equality below is a
    // fact about the derive, not a round-trip law - Display output still
    // must never be parsed.
    let value = sample();
    assert_eq!(
        value.print_to_string(FolioMode::Display),
        value.print_to_string(FolioMode::Full)
    );
}

#[test]
fn the_budget_observer_page_holds_the_full_mode_laws() {
    let budget = BudgetObserver::parse(BUDGET_CANONICAL).expect("canonical text parses");
    assert_eq!(
        budget.print_to_string(FolioMode::Full).as_str(),
        BUDGET_CANONICAL
    );
    let reparsed = BudgetObserver::parse(budget.print_to_string(FolioMode::Full).as_str())
        .expect("printed text parses");
    assert_eq!(reparsed, budget);
    assert_eq!(
        budget,
        BudgetObserver {
            walks: 2,
            passes: 3,
            analyses: 0,
            pipelines: 1,
            failures: 0,
        }
    );
}

fn parse_err(input: &str) -> FolioError {
    SamplePage::parse(input).expect_err("input must not parse")
}

#[test]
fn parse_errors_carry_line_numbers_and_exact_messages() {
    assert_eq!(
        parse_err("x\n"),
        FolioError::new(1, cstr!("content before the [sample-page] header"))
    );
    assert_eq!(
        parse_err("[sample-page.notes]\n"),
        FolioError::new(1, cstr!("first section must be [sample-page]"))
    );
    assert_eq!(
        parse_err(""),
        FolioError::new(0, cstr!("missing [sample-page] header"))
    );
    assert_eq!(
        parse_err("[sample-page]\ntitle=a\nenabled=true\ncount=1\n\n[sample-page]\n"),
        FolioError::new(6, cstr!("duplicate section [sample-page]"))
    );
    assert_eq!(
        parse_err("[sample-page]\n\n[sample-page.notes]\na\n\n[sample-page.notes]\n"),
        FolioError::new(6, cstr!("duplicate section [sample-page.notes]"))
    );
    assert_eq!(
        parse_err("[sample-page]\n\n[sample-page.bogus]\n"),
        FolioError::new(3, cstr!("unknown section [sample-page.bogus]"))
    );
    assert_eq!(
        parse_err("[sample-page]\n\n[other]\n"),
        FolioError::new(3, cstr!("unknown section [other]"))
    );
    assert_eq!(
        parse_err("[sample-page]\nbogus=1\n"),
        FolioError::new(2, cstr!("unknown field `bogus`"))
    );
    assert_eq!(
        parse_err("[sample-page]\ntitle=a\ntitle=b\n"),
        FolioError::new(3, cstr!("duplicate field `title`"))
    );
    assert_eq!(
        parse_err("[sample-page]\ntitle=a\nenabled=true\n"),
        FolioError::new(0, cstr!("missing field `count`"))
    );
    assert_eq!(
        parse_err("[sample-page]\njusttext\n"),
        FolioError::new(2, cstr!("field line is missing `=`"))
    );
    assert_eq!(
        parse_err("[sample-page]\nenabled=yes\n"),
        FolioError::new(2, cstr!("invalid bool `yes`"))
    );
    assert_eq!(
        parse_err("[sample-page]\ncount=x\n"),
        FolioError::new(2, cstr!("invalid integer `x`"))
    );
    assert_eq!(
        parse_err(
            "[sample-page]\ntitle=a\nenabled=true\ncount=1\n\n[sample-page.weights]\nnoequals\n"
        ),
        FolioError::new(7, cstr!("map entry line is missing `=`"))
    );
    assert_eq!(
        parse_err(
            "[sample-page]\ntitle=a\nenabled=true\ncount=1\n\n[sample-page.weights]\nalpha=1\nalpha=2\n"
        ),
        FolioError::new(8, cstr!("duplicate map key `alpha`"))
    );
}
