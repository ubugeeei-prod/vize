use super::*;
use crate::{StageConfig, compute_file_artifacts, parse_descriptor};
use core::ptr;

const SOURCE: &str = "<style module lang='scss'>.card { &-label {color:red} }</style>\n<script setup>const n=1;</script>\n<template><div>{{n}}</div></template><style module='theme'>.雪{}</style><docs>notes</docs>";

#[test]
fn every_stage_and_file_position_borrows_the_original_block() {
    let db = ResidentDatabase::default();
    let file = db.open("Card.vue", SOURCE);
    let view = db.document_blocks(file).unwrap();
    let clean = compute_file_artifacts(SOURCE, StageConfig::default());
    assert_eq!(view.iter().len(), clean.len());
    for (block, fresh) in view.iter().zip(&clean) {
        assert_eq!(
            (block.kind(), block.ordinal(), block.start()),
            (&fresh.kind, fresh.ordinal, fresh.start)
        );
        assert_eq!(block.source().key, fresh.source_key);
        assert_eq!(block.surface(), fresh.surface.as_ref());
        assert_eq!(block.page(), fresh.page.as_ref());
        if let (Some(a), Some(b)) = (block.page(), block.page()) {
            assert!(ptr::eq(a, b));
        }
    }
    for i in 0..=SOURCE.len() {
        let offset = i as u32;
        if !SOURCE.is_char_boundary(i) {
            assert_eq!(
                view.at(offset).err(),
                Some(BlockPositionError::NotCharBoundary)
            );
            continue;
        }
        let expected = view.iter().find(|block| {
            offset >= block.start()
                && ((offset - block.start()) as usize) < block.source().text.len()
        });
        assert_eq!(
            view.at(offset).unwrap().map(BlockView::start),
            expected.map(BlockView::start)
        );
    }
    assert_eq!(
        view.at(u32::MAX).err(),
        Some(BlockPositionError::OutOfBounds)
    );
    assert_eq!(
        view.iter()
            .filter_map(BlockView::style_module)
            .collect::<Vec<_>>(),
        ["$style", "theme"]
    );
}

#[test]
fn shifts_reuse_the_canonical_artifacts_and_edited_blocks_match_clean() {
    let mut db = ResidentDatabase::default();
    let file = db.open("Card.vue", SOURCE);
    let start = {
        let view = db.document_blocks(file).unwrap();
        let first = view.iter().next().unwrap();
        assert!(first.page().is_some());
        first.start()
    };
    let demanded = db.take_accounting();
    assert_eq!(demanded["s2_page"].executed, 1);
    assert!(!demanded.contains_key("s1_block"));
    let shifted = vize_l0::cstr!("<!-- 雪 -->{SOURCE}");
    db.edit(file, &shifted);
    let view = db.document_blocks(file).unwrap();
    let first = view.iter().next().unwrap();
    assert_eq!(first.start(), start + "<!-- 雪 -->".len() as u32);
    assert_eq!(
        first
            .authored_span(Span::new(0, 5))
            .unwrap()
            .slice(&shifted),
        ".card"
    );
    assert!(first.page().is_some());
    assert_eq!(
        db.take_accounting()
            .get("s2_page")
            .map_or(0, |row| row.executed),
        0
    );
    let changed = shifted.replace("card", "pane");
    db.edit(file, &changed);
    let view = db.document_blocks(file).unwrap();
    let first = view.iter().next().unwrap();
    assert_eq!(
        first.page(),
        compute_file_artifacts(&changed, StageConfig::default())[0]
            .page
            .as_ref()
    );
    assert_eq!(db.take_accounting()["s2_page"].executed, 1);
}

#[test]
fn a_rejected_partition_keeps_the_original_descriptor_diagnostic() {
    let db = ResidentDatabase::default();
    let bad = "<script setup>1</script><script setup>2</script>";
    assert!(parse_descriptor("Bad.vue", bad).is_none());
    let file = db.open("Bad.vue", bad);
    let error = db.document_blocks(file).err().unwrap();
    let ParsedSfc::Failed(original) = sfc_descriptor(&db, file) else {
        panic!("descriptor error")
    };
    assert!(ptr::eq(error, original));
}

#[test]
fn positions_require_no_stages_and_configuration_only_invalidates_the_page() {
    let mut db = ResidentDatabase::default();
    let file = db.open("Card.vue", SOURCE);
    let view = db.document_blocks(file).unwrap();
    let byte = SOURCE.find("card").unwrap() as u32;
    assert_eq!(view.at(byte).unwrap().unwrap().kind(), &BlockKind::Style);
    let first = db.take_accounting();
    assert_eq!(first["sfc_blocks"].executed, 1);
    assert_eq!(first.len(), 1);
    let style = view.at(byte).unwrap().unwrap();
    assert!(style.page().is_some());
    let _ = db.take_accounting();
    db.configure(StageConfig {
        vue_version: vize_l0::config::VueVersion::V2,
    });
    let view = db.document_blocks(file).unwrap();
    let style = view.at(byte).unwrap().unwrap();
    assert!(style.page().is_some());
    let after = db.take_accounting();
    assert_eq!(after["s2_page"].executed, 1);
    assert!(!after.contains_key("s1_block"));
}

#[test]
fn mapped_spans_preserve_unicode_and_empty_boundaries() {
    let db = ResidentDatabase::default();
    let text = "<docs>雪</docs><style module='theme'></style>";
    let file = db.open("Styles.vue", text);
    let view = db.document_blocks(file).unwrap();
    let docs = view.iter().next().unwrap();
    let empty_style = view.iter().last().unwrap();
    assert_eq!(empty_style.style_module(), Some("theme"));
    assert!(view.at(empty_style.start()).unwrap().is_none());
    assert_eq!(
        docs.authored_span(Span::new(0, 3)).unwrap().slice(text),
        "雪"
    );
    assert!(docs.authored_span(Span::new(1, 3)).is_none());
    assert!(docs.authored_span(Span::new(0, 4)).is_none());
    assert!(docs.authored_span(Span::new(2, 1)).is_none());
    let empty = docs.authored_span(Span::new(3, 3)).unwrap();
    assert!(empty.is_empty() && view.at(empty.start).unwrap().is_none());
}

#[test]
fn half_open_lookup_matches_a_linear_scan_across_short_and_long_partitions() {
    for length in [5, 6, 7, 16] {
        let mut source = vize_l0::String::default();
        for i in 0..length {
            let body = if i % 3 == 0 { "" } else { "雪x" };
            source.push_str("<note>");
            source.push_str(body);
            source.push_str("</note>");
        }
        let db = ResidentDatabase::default();
        let file = db.open("Notes.vue", &source);
        let view = db.document_blocks(file).unwrap();
        assert_eq!(view.iter().len(), length);
        for byte in 0..=source.len() {
            let at = view.at(byte as u32);
            if !source.is_char_boundary(byte) {
                assert_eq!(at.err(), Some(BlockPositionError::NotCharBoundary));
                continue;
            }
            let expected = view.iter().find(|r| {
                byte.checked_sub(r.start() as usize)
                    .is_some_and(|rel| rel < r.source().text.len())
            });
            assert_eq!(
                at.unwrap().map(BlockView::ordinal),
                expected.map(BlockView::ordinal)
            );
        }
    }
}
