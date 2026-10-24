//! The stack-of-open-elements walk every scope and loop scan is built from.
//!
//! This kernel depends on nothing but [`Tri`], so the Kani harness in
//! `crates/vize_patina/src/html_content_model/kani.rs` verifies this exact source:
//! the walk is sound for every concrete stack a possibly truncated chain
//! describes, and a definite verdict on a truncated chain survives any
//! ancestors the context later reveals.

use super::tri::Tri;

/// How a frame participates in a scan under one namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    /// The element the scan looks for.
    Target,
    /// A scope boundary, a special element or a marker: the scan ends here.
    Stop,
    /// Neither: the scan continues with the next open element.
    Pass,
}

/// What one frame contributes to a scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Step {
    /// The frame decides the scan.
    Decided(Tri),
    /// The scan continues below the frame.
    Pass,
}

/// Classify one frame from its role under each namespace it may have.
///
/// A frame with no possible namespace passes. A frame whose role differs
/// across its possible namespaces makes the scan unknown.
pub(super) fn classify(mut roles: impl Iterator<Item = Role>) -> Step {
    let first = roles.next().unwrap_or(Role::Pass);
    if roles.any(|other| other != first) {
        return Step::Decided(Tri::Maybe);
    }
    match first {
        Role::Target => Step::Decided(Tri::Yes),
        Role::Stop => Step::Decided(Tri::No),
        Role::Pass => Step::Pass,
    }
}

/// Walk `frames` from the innermost (last) outwards. `roles` yields a
/// frame's role under each of its possible namespaces, and `exhausted` is
/// the verdict when every known frame passes.
///
/// Returns the verdict and, when it is `Yes`, the index of the deciding
/// frame.
pub(super) fn walk<'a, F, I>(
    frames: &'a [F],
    exhausted: Tri,
    roles: impl Fn(&'a F) -> I,
) -> (Tri, Option<usize>)
where
    I: Iterator<Item = Role>,
{
    for (index, frame) in frames.iter().enumerate().rev() {
        match classify(roles(frame)) {
            Step::Decided(Tri::Yes) => return (Tri::Yes, Some(index)),
            Step::Decided(verdict) => return (verdict, None),
            Step::Pass => {}
        }
    }
    (exhausted, None)
}
