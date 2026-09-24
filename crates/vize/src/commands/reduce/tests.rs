//! Unit laws for the reducer's two halves: the deletion vocabulary's exact
//! ranges and tree shape, and the driver's 1-minimality claim, checked
//! independently of the driver's own bookkeeping.
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use std::ops::Range;

use super::driver::{ReduceError, reduce};
use super::vocabulary::{candidates, delete};

const SEEDED: &str = include_str!("../../../tests/fixtures/davinci_reduce/seeded-crash.vue");

#[test]
fn candidates_are_the_exact_s1_subtrees_in_pre_order() {
    let source = "<div a=\"1\"> x <b/></div>\n";
    let ranges = candidates(source);
    let slices: Vec<&str> = ranges.iter().map(|range| &source[range.clone()]).collect();
    // The element, its attribute (leading space included), its text, the
    // nested element, then the trailing newline text.
    assert_eq!(
        slices,
        [
            "<div a=\"1\"> x <b/></div>",
            " a=\"1\"",
            " x ",
            "<b/>",
            "\n"
        ]
    );
    assert_eq!(delete(source, &ranges[1..2]), "<div> x <b/></div>\n");
    assert_eq!(delete(source, &ranges[0..2]), "\n");
}

#[test]
fn candidate_ranges_nest_like_a_tree() {
    let ranges = candidates(SEEDED);
    // Census pin: the fixture's S1 nodes plus attributes; a vocabulary change
    // moves it deliberately.
    assert_eq!(ranges.len(), 92);
    for (index, a) in ranges.iter().enumerate() {
        assert!(SEEDED.is_char_boundary(a.start) && SEEDED.is_char_boundary(a.end));
        for b in &ranges[index + 1..] {
            let disjoint = a.end <= b.start || b.end <= a.start;
            let nested = contains(a, b) || contains(b, a);
            assert!(
                disjoint || nested,
                "{a:?} and {b:?} overlap without nesting"
            );
        }
    }
}

fn contains(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

#[test]
fn a_reduction_is_one_minimal_and_keeps_the_oracle() {
    let predicate = |text: &str| text.contains("<seed-crash") && text.contains("slides");
    let mut calls = 0;
    let reduction = reduce(SEEDED, 10_000, &mut |text| {
        calls += 1;
        predicate(text)
    })
    .expect("the input is interesting");
    assert_eq!(reduction.runs, calls);
    assert!(!reduction.exhausted);
    assert!(predicate(&reduction.text));
    // 1-minimality, re-derived: no single remaining candidate can go.
    for range in candidates(&reduction.text) {
        let smaller = delete(&reduction.text, std::slice::from_ref(&range));
        assert!(!predicate(&smaller), "{range:?} could still be deleted");
    }
    assert_eq!(
        reduction.text.as_str(),
        "<template><section><Swiper><SwiperSlide v-for=\"slide in slides\"><article>\
         <seed-crash /></article></SwiperSlide></Swiper></section></template>"
    );
}

#[test]
fn an_uninteresting_input_is_refused_before_any_deletion() {
    let mut calls = 0;
    let outcome = reduce(SEEDED, 10_000, &mut |_| {
        calls += 1;
        false
    });
    assert_eq!(outcome, Err(ReduceError::NotInteresting));
    assert_eq!(calls, 1);
}

#[test]
fn the_run_budget_stops_the_driver_with_a_valid_artifact() {
    let reduction =
        reduce(SEEDED, 3, &mut |text| text.contains("<seed-crash")).expect("interesting");
    assert!(reduction.exhausted);
    assert_eq!(reduction.runs, 3);
    assert!(reduction.text.contains("<seed-crash"));
}
