//! The shared fix selection policy and same-point insertions.

use super::{EditError, EditSet, VersionedSource, select_first};
use vize_l0::{SourceRoot, Span};

fn snapshot(source: &str) -> VersionedSource<'_, (&str, u64)> {
    VersionedSource::new(("fix pass", 1), 0, SourceRoot::new(source).unwrap())
}

#[test]
fn insertions_at_one_point_apply_in_supplied_order() {
    let frame = snapshot("ab");
    let edits = [
        frame.edit(Span::new(1, 1), "x").unwrap(),
        frame.edit(Span::new(1, 1), "y").unwrap(),
        frame.edit(Span::new(1, 2), "B").unwrap(),
        frame.edit(Span::new(2, 2), "!").unwrap(),
        frame.edit(Span::new(2, 2), "?").unwrap(),
    ];
    let set = EditSet::new(frame, &edits).unwrap();
    assert_eq!(set.apply(frame).unwrap(), "axyB!?");
    // A replacement still consumes its start: no insertion may return to it.
    let back = [
        frame.edit(Span::new(0, 1), "A").unwrap(),
        frame.edit(Span::new(0, 0), "x").unwrap(),
    ];
    assert_eq!(EditSet::new(frame, &back).unwrap_err(), EditError::Overlap);
}

#[test]
fn selection_sorts_stably_and_keeps_the_first_of_overlapping_edits() {
    let source = "abcdefgh";
    let frame = snapshot(source);
    let mut edits = [
        frame.edit(Span::new(5, 7), "late").unwrap(),
        frame.edit(Span::new(1, 3), "first").unwrap(),
        frame.edit(Span::new(2, 4), "inside").unwrap(),
        frame.edit(Span::new(3, 3), "p").unwrap(),
        frame.edit(Span::new(3, 3), "q").unwrap(),
        frame.edit(Span::new(1, 3), "again").unwrap(),
        frame.edit(Span::new(6, 6), "within").unwrap(),
        frame.edit(Span::new(0, 0), "<").unwrap(),
    ];
    let kept = select_first(&mut edits);
    let selected = edits.get(..kept).unwrap();
    assert!(
        selected
            .iter()
            .map(|edit| edit.replacement())
            .eq(["<", "first", "p", "q", "late"])
    );
    let set = EditSet::new(frame, selected).unwrap();
    assert_eq!(set.apply(frame).unwrap(), "<afirstpqdelateh");
    assert_eq!(select_first::<(&str, u64)>(&mut []), 0);
}

/// The selection reproduces the splice loop products ran before sharing it.
#[test]
fn selection_matches_the_previous_splice_from_the_end_applier() {
    let source = "<div  class=x >é</div>";
    let frame = snapshot(source);
    let raw: [(u32, u32, &str); 7] = [
        (4, 6, " "),
        (12, 13, "\"x\""),
        (12, 13, "'x'"),
        (14, 14, ""),
        (13, 13, "/"),
        (13, 13, "*"),
        (0, 0, "\n"),
    ];
    let mut expected = vize_l0::String::from(source);
    let mut sorted = raw;
    sorted.sort_by_key(|&(start, end, _)| (start, end));
    let mut chosen = [false; 7];
    let mut last_end = 0;
    for (keep, edit) in chosen.iter_mut().zip(sorted) {
        if edit.0 >= last_end {
            last_end = edit.1;
            *keep = true;
        }
    }
    for (keep, (start, end, text)) in chosen.into_iter().zip(sorted).rev() {
        if keep {
            expected.replace_range(start as usize..end as usize, text);
        }
    }
    let mut edits = raw.map(|(start, end, text)| frame.edit(Span::new(start, end), text).unwrap());
    let kept = select_first(&mut edits);
    let set = EditSet::new(frame, edits.get(..kept).unwrap()).unwrap();
    assert_eq!(set.apply(frame).unwrap(), expected);
}
