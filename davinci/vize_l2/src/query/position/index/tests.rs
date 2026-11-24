//! Compare every published position answer against the original scanning model.

use super::*;
use crate::query::{TemplateSiteRef, TemplateSiteScope};
use core::ptr;

pub(in crate::query) fn assert_equivalent(file: &FileArtifact<'_>) {
    let scan = Model::new(file);
    let mut index = scan.index_positions();
    compare(scan, index.model());
    // Exercise indexed lookup even on the small fixtures used for edge cases.
    if file.is_complete() && !index.rows.indexed_script {
        index.rows.index_script(file);
        compare(scan, index.model());
    }
}

fn compare(scan: Model<'_, '_>, indexed: IndexedModel<'_, '_>) {
    let file = scan.file();
    for byte in (0..=file.artifact().source().len() as u32).chain([u32::MAX]) {
        let combined = indexed.script_position_at_offset(byte);
        for actual in [indexed.reference_at_offset(byte), combined.reference] {
            assert_eq!(
                scan.reference_at_offset(byte)
                    .map(|r| r.map(|x| (ptr::from_ref(x.file()), ptr::from_ref(x.reference())))),
                actual.map(|r| r.map(|x| (ptr::from_ref(x.file()), ptr::from_ref(x.reference())))),
                "reference {byte}"
            );
        }
        for actual in [indexed.binding_at_offset(byte), combined.binding] {
            assert_eq!(
                scan.binding_at_offset(byte)
                    .map(|r| r.map(|x| (ptr::from_ref(x.file()), x.id()))),
                actual.map(|r| r.map(|x| (ptr::from_ref(x.file()), x.id()))),
                "binding {byte}"
            );
        }
        for actual in [indexed.scope_at_offset(byte), combined.scope] {
            assert_eq!(
                scan.scope_at_offset(byte)
                    .map(|r| r.map(|x| (ptr::from_ref(x.file()), ptr::from_ref(x.scope())))),
                actual.map(|r| r.map(|x| (ptr::from_ref(x.file()), ptr::from_ref(x.scope())))),
                "scope {byte}"
            );
        }
        match (
            scan.template_symbol_at_offset(byte),
            indexed.template_symbol_at_offset(byte),
        ) {
            (Ok(Some(expected)), Ok(Some(actual))) => same_site(expected, actual),
            (Ok(None), Ok(None)) => {}
            (Err(expected), Err(actual)) => assert_eq!(actual, expected, "template error {byte}"),
            _ => panic!("different template answer at {byte}"),
        }
    }
}

fn same_site(expected: TemplateSiteRef<'_, '_>, actual: TemplateSiteRef<'_, '_>) {
    assert!(ptr::eq(actual.file(), expected.file()));
    assert_eq!(actual.span(), expected.span());
    assert_eq!(actual.usage(), expected.usage());
    assert!(actual.symbol().same_symbol(expected.symbol()));
    assert_eq!(
        actual.handler().map(|h| h.id()),
        expected.handler().map(|h| h.id())
    );
    assert_eq!(
        actual.for_head().map(|h| h.id()),
        expected.for_head().map(|h| h.id())
    );
    match (actual.scope(), expected.scope()) {
        (TemplateSiteScope::File(a), TemplateSiteScope::File(e)) => assert_eq!(a, e),
        (TemplateSiteScope::Handler(a, ascope), TemplateSiteScope::Handler(e, escope)) => {
            assert!(a.same_owner(e));
            assert_eq!((a.id(), ascope), (e.id(), escope));
        }
        _ => panic!("different template scopes"),
    }
}

#[test]
fn interval_tree_preserves_duplicate_and_crossing_spans() {
    use vize_l0::Span;
    let spans = [
        (1, 6),
        (0, 15),
        (3, 9),
        (3, 9),
        (10, 12),
        (6, 6),
        (8, 3),
        (0, 1),
        (14, 16),
        (12, 15),
    ];
    let index = Intervals::from_rows(&spans, |&(start, end)| Span::new(start, end));
    for at in 0..=17 {
        let mut seen = 0u32;
        index
            .visit(at, &mut |_, id| {
                seen |= 1 << id;
                Ok::<_, ()>(())
            })
            .unwrap();
        let mask = spans
            .iter()
            .enumerate()
            .filter(|(_, range)| range.0 <= at && at < range.1)
            .fold(0, |mask, (i, _)| mask | (1 << i));
        assert_eq!(seen, mask);
        let result = index.unique(at, |_, index| Some(*index));
        if mask.count_ones() > 1 {
            assert_eq!(result, Err(PositionQueryError::AmbiguousSite));
        } else {
            assert_eq!(
                result,
                Ok((mask != 0).then(|| mask.trailing_zeros() as usize))
            );
        }
    }
}
