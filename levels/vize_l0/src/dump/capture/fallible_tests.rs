use core::{cell::Cell, fmt};

use super::{CaptureOutcome, CaptureSink, NoCapture, StageCapture};
use crate::{String, level::Level};

struct InspectionError<'a> {
    formatted: &'a Cell<u32>,
    dropped: &'a Cell<u32>,
}

impl fmt::Display for InspectionError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.formatted.set(self.formatted.get() + 1);
        formatter.write_str("unsupported native binding at bytes 4..9: λ")
    }
}

impl Drop for InspectionError<'_> {
    fn drop(&mut self) {
        self.dropped.set(self.dropped.get() + 1);
    }
}

#[test]
fn ordinary_capture_never_renders_or_formats_a_failure() {
    let rendered = Cell::new(0);
    let formatted = Cell::new(0);
    let dropped = Cell::new(0);
    let error = InspectionError {
        formatted: &formatted,
        dropped: &dropped,
    };
    NoCapture.try_page(Level::L2, "lower", || {
        rendered.set(rendered.get() + 1);
        Err(error)
    });
    assert_eq!(core::mem::size_of::<NoCapture>(), 0);
    assert_eq!(rendered.get(), 0);
    assert_eq!(formatted.get(), 0);
    assert_eq!(dropped.get(), 1);
}

#[test]
fn success_renders_once_and_keeps_the_existing_page_contract() {
    let mut capture = StageCapture::new("dom");
    let rendered = Cell::new(0);
    capture.page(Level::L1, "parse", || String::from("authored syntax"));
    capture.try_page(Level::L2, "lower", || {
        rendered.set(rendered.get() + 1);
        Ok::<_, &'static str>(String::from("canonical ops\n"))
    });
    capture.finish(|| CaptureOutcome::Accepted);
    assert_eq!(rendered.get(), 1);
    assert_eq!(capture.pages.len(), 2);
    assert_eq!(
        capture.pages.last().map(|page| page.text.as_str()),
        Some("canonical ops\n")
    );
    assert!(capture.unavailable.is_empty());
}

#[test]
fn failure_keeps_its_full_reason_and_drops_the_original_error_once() {
    let mut capture = StageCapture::new("dom");
    let rendered = Cell::new(0);
    let formatted = Cell::new(0);
    let dropped = Cell::new(0);
    capture.try_page(Level::L2, "lower", || {
        rendered.set(rendered.get() + 1);
        Err(InspectionError {
            formatted: &formatted,
            dropped: &dropped,
        })
    });
    assert_eq!(rendered.get(), 1);
    assert_eq!(formatted.get(), 1);
    assert_eq!(dropped.get(), 1);
    assert!(capture.pages.is_empty());
    let failure = capture.unavailable.first();
    assert_eq!(failure.map(|failure| failure.level), Some(Level::L2));
    assert_eq!(failure.map(|failure| failure.step), Some("lower"));
    assert_eq!(
        failure.map(|failure| failure.reason.as_str()),
        Some("unsupported native binding at bytes 4..9: λ")
    );
    capture.finish(|| CaptureOutcome::Accepted);
    assert_eq!(capture.unavailable.len(), 1);
}

#[test]
fn a_failed_boundary_does_not_replace_successful_later_pages() {
    let mut capture = StageCapture::new("dom");
    capture.try_page(Level::L2, "lower", || {
        Err::<String, _>("native binding unsupported")
    });
    capture.page(Level::L4, "emit", || String::from("module"));
    capture.finish(|| CaptureOutcome::Accepted);
    assert_eq!(capture.pages.len(), 1);
    assert_eq!(
        capture.pages.first().map(|page| page.level),
        Some(Level::L4)
    );
    assert_eq!(capture.unavailable.len(), 1);
}

#[test]
fn every_nonaccepted_outcome_discards_provisional_failed_inspections() {
    for outcome in [
        CaptureOutcome::Pending,
        CaptureOutcome::Legacy(String::from("compatibility selected")),
        CaptureOutcome::Unavailable(String::from("no template")),
        CaptureOutcome::Rejected(String::from("invalid artifact")),
    ] {
        let mut capture = StageCapture::new("dom");
        capture.page(Level::L1, "parse", || String::from("syntax"));
        capture.try_page(Level::L2, "lower", || {
            Err::<String, _>("inspection unavailable")
        });
        capture.finish(|| outcome.clone());
        assert_eq!(capture.outcome, outcome);
        assert!(capture.pages.is_empty());
        assert!(capture.unavailable.is_empty());
    }
}
