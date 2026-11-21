use super::*;
use vize_l0::{SourceRoot, String};

fn snapshot(text: &str) -> VersionedSource<'_, (&str, u64)> {
    VersionedSource::new(("doc", 4), 3, SourceRoot::new(text).unwrap())
}

#[test]
fn same_offset_insertions_preserve_diagnostic_order_and_replacement_boundaries() {
    let source = snapshot("ab");
    let candidates = [
        (Span::new(0, 2), "long"),
        (Span::new(0, 0), "1"),
        (Span::new(0, 1), "X"),
        (Span::new(0, 1), "loses tie"),
        (Span::new(1, 1), "3"),
        (Span::new(0, 0), "2"),
        (Span::new(2, 2), "!"),
    ];
    let selected = source.select_edits(candidates).unwrap().unwrap();
    let set = selected.as_edit_set();
    assert!(
        set.edits()
            .iter()
            .map(|edit| edit.replacement())
            .eq(["1", "2", "X", "3", "!"])
    );
    assert_eq!(set.apply(source).unwrap(), "12X3b!");
    let strict = [
        source.edit(Span::new(0, 0), "1").unwrap(),
        source.edit(Span::new(0, 0), "2").unwrap(),
    ];
    assert_eq!(
        EditSet::new(source, &strict).unwrap_err(),
        EditError::Overlap
    );
}

#[test]
fn invalid_utf8_reversed_and_out_of_range_edits_never_displace_a_valid_edit() {
    let source = snapshot("α🙂z");
    let candidates = [
        (Span::new(1, 2), "a"),
        (Span::new(6, 2), "b"),
        (Span::new(3, 3), "c"),
        (Span::new(0, u32::MAX), "d"),
        (Span::new(2, 6), "🌱"),
    ];
    let selected = source.select_edits(candidates).unwrap().unwrap();
    let set = selected.as_edit_set();
    assert_eq!(set.edits().len(), 1);
    assert_eq!(set.apply(source).unwrap(), "α🌱z");
    assert!(source.select_edits([]).unwrap().is_none());
    assert!(
        source
            .select_edits(candidates[..4].iter().copied())
            .unwrap()
            .is_none()
    );
}

#[test]
fn publication_and_application_recheck_version_key_and_buffer() {
    let text = String::from("original source is long enough for distinct physical backed buffers");
    let copy = text.clone();
    let source = snapshot(&text);
    let stale = VersionedSource::new(source.key(), 2, source.root());
    let reopened = VersionedSource::new(("doc", 5), 3, source.root());
    let selected = source
        .select_edits([(Span::new(0, 0), "!")])
        .unwrap()
        .unwrap();
    let set = selected.as_edit_set();
    for (current, error) in [
        (stale, EditError::StaleVersion),
        (reopened, EditError::SnapshotMismatch),
        (snapshot(&copy), EditError::SourceMismatch),
    ] {
        assert_eq!(set.check_current(current), Err(error));
        assert_eq!(set.apply(current), Err(error));
    }
}
