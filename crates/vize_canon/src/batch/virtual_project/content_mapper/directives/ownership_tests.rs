//! Genuine b3 mapping geometry: a wrapper cannot acquire an echo's EXPECT.
use super::{ContentMapperSpan, mapped_virtual_ranges};
use crate::virtual_ts::diagnostic_ownership::DiagnosticEcho;

fn echo() -> DiagnosticEcho {
    DiagnosticEcho {
        generated: 21029..21044,
        authored: 157..172,
        suppress: true,
    }
}

const NAVIGATION: [ContentMapperSpan; 3] = [
    ContentMapperSpan([21028, 1, 157, 15, 1, 0]),
    ContentMapperSpan([21029, 15, 157, 15, 0, 1048575]),
    ContentMapperSpan([21044, 1, 157, 15, 1, 0]),
];

#[test]
fn exact_copy_owner_also_owns_its_adjacent_diagnostic_wrappers() {
    assert!(mapped_virtual_ranges(&NAVIGATION, &[157..172], &[echo()]).is_empty());
    assert_eq!(
        mapped_virtual_ranges(&NAVIGATION, &[157..172], &[]),
        vec![21028..21045],
        "without an emitter ownership proof every mapping remains eligible"
    );
}

#[test]
fn foreign_owner_cannot_claim_adjacent_wrapper_atoms() {
    let mut foreign = echo();
    foreign.authored = 158..173;
    assert_eq!(
        mapped_virtual_ranges(&NAVIGATION, &[157..172], &[foreign]),
        vec![21028..21029, 21044..21045]
    );
}

#[test]
fn nonadjacent_and_editor_capable_mappings_stay_independent() {
    let rows = [
        ContentMapperSpan([21026, 1, 157, 15, 1, 0]),
        ContentMapperSpan([21044, 1, 157, 15, 0, 1048575]),
    ];
    assert_eq!(
        mapped_virtual_ranges(&rows, &[157..172], &[echo()]),
        vec![21026..21027, 21044..21045]
    );
}
