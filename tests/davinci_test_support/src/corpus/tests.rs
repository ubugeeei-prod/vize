//! Synthetic inventory witnesses: every rejection class the fail-closed
//! corpus gate promises, pinned without a real repository.

use std::collections::BTreeMap;

use vize_s0::{CompactString, cstr};

use super::{
    CANONICAL_CORPUS_RELATIVE, InventoryError, SubmoduleState, is_canonical_root,
    parse_indexed_gitlinks, parse_submodule_status, reconcile,
};

const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

fn stage_record(path: &str) -> CompactString {
    cstr!("160000 {SHA} 0\t{path}\0")
}

fn status_line(marker: char, path: &str) -> CompactString {
    cstr!("{marker}{SHA} {path}\n")
}

fn parse_pair(
    stage: &str,
    status: &str,
) -> (
    super::IndexedGitlinks,
    BTreeMap<CompactString, SubmoduleState>,
) {
    (
        parse_indexed_gitlinks(stage).expect("index parses"),
        parse_submodule_status(status).expect("status parses"),
    )
}

#[test]
fn clean_inventory_reconciles() {
    let stage = cstr!(
        "{}{}",
        stage_record("tests/_fixtures/_git/alpha"),
        stage_record("tests/_fixtures/_git/beta")
    );
    let status = cstr!(
        "{}{}",
        status_line(' ', "tests/_fixtures/_git/alpha (v1.0.0)"),
        status_line(' ', "tests/_fixtures/_git/beta (heads/main)")
    );
    let (indexed, states) = parse_pair(&stage, &status);
    assert_eq!(reconcile(&indexed, &states), Ok(2));
}

#[test]
fn partial_hydration_is_pinned_exactly() {
    // The synthetic canonical shape: 146 indexed gitlinks, 5 hydrated clean,
    // 141 missing. This must fail closed with the exact diagnostic below.
    let mut stage = CompactString::default();
    let mut status = CompactString::default();
    for index in 0..146u32 {
        let path = cstr!("tests/_fixtures/_git/project-{index:03}");
        stage.push_str(&stage_record(&path));
        if index < 5 {
            status.push_str(&status_line(' ', &cstr!("{path} (v1.0.0)")));
        } else {
            status.push_str(&status_line('-', &path));
        }
    }
    let (indexed, states) = parse_pair(&stage, &status);
    assert_eq!(indexed.total(), 146);
    let error = reconcile(&indexed, &states).unwrap_err();
    assert_eq!(
        error,
        InventoryError::Missing {
            missing: 141,
            indexed: 146,
            first: "tests/_fixtures/_git/project-005".into(),
        }
    );
    assert_eq!(
        cstr!("{error}"),
        "differential corpus is not closure evidence: 141 of 146 indexed fixture submodules \
         are missing (unhydrated); first missing `tests/_fixtures/_git/project-005`; hydrate \
         with `git submodule update --init --checkout`"
    );
}

#[test]
fn drifted_submodules_are_rejected() {
    let stage = stage_record("tests/_fixtures/_git/alpha");
    let status = status_line('+', "tests/_fixtures/_git/alpha (v1.0.0-2-gabcdef0)");
    let (indexed, states) = parse_pair(&stage, &status);
    assert_eq!(
        reconcile(&indexed, &states),
        Err(InventoryError::Drifted {
            drifted: 1,
            indexed: 1,
            first: "tests/_fixtures/_git/alpha".into(),
        })
    );
}

#[test]
fn conflicted_status_is_rejected() {
    let stage = stage_record("tests/_fixtures/_git/alpha");
    let status = status_line('U', "tests/_fixtures/_git/alpha");
    let (indexed, states) = parse_pair(&stage, &status);
    assert_eq!(
        reconcile(&indexed, &states),
        Err(InventoryError::Conflicted {
            conflicted: 1,
            indexed: 1,
            first: "tests/_fixtures/_git/alpha".into(),
        })
    );
}

#[test]
fn conflicted_index_stages_are_rejected() {
    let stage = cstr!(
        "160000 {SHA} 1\ttests/_fixtures/_git/alpha\0160000 {SHA} 2\ttests/_fixtures/_git/alpha\0"
    );
    let status = status_line(' ', "tests/_fixtures/_git/alpha (v1.0.0)");
    let (indexed, states) = parse_pair(&stage, &status);
    assert_eq!(indexed.conflicted.len(), 1);
    assert_eq!(
        reconcile(&indexed, &states),
        Err(InventoryError::Conflicted {
            conflicted: 1,
            indexed: 1,
            first: "tests/_fixtures/_git/alpha".into(),
        })
    );
}

#[test]
fn unknown_status_markers_are_rejected() {
    let status = status_line('?', "tests/_fixtures/_git/alpha");
    assert_eq!(
        parse_submodule_status(&status),
        Err(InventoryError::UnknownMarker {
            marker: '?',
            path: "tests/_fixtures/_git/alpha".into(),
        })
    );
}

#[test]
fn paths_with_spaces_survive_both_parsers() {
    let stage = stage_record("tests/_fixtures/_git/my project");
    let status = cstr!(
        "{}{}",
        status_line(' ', "tests/_fixtures/_git/my project (v1.0.0)"),
        ""
    );
    let (indexed, states) = parse_pair(&stage, &status);
    assert!(indexed.clean.contains("tests/_fixtures/_git/my project"));
    assert_eq!(
        states.get("tests/_fixtures/_git/my project"),
        Some(&SubmoduleState::Clean)
    );
    assert_eq!(reconcile(&indexed, &states), Ok(1));
}

#[test]
fn missing_paths_with_spaces_are_not_describe_stripped() {
    // `-` entries never carry a describe suffix; a path that happens to end
    // in ` (x)` must survive verbatim.
    let stage = stage_record("tests/_fixtures/_git/odd (1)");
    let status = status_line('-', "tests/_fixtures/_git/odd (1)");
    let (indexed, states) = parse_pair(&stage, &status);
    assert_eq!(
        reconcile(&indexed, &states),
        Err(InventoryError::Missing {
            missing: 1,
            indexed: 1,
            first: "tests/_fixtures/_git/odd (1)".into(),
        })
    );
}

#[test]
fn empty_inventory_is_rejected() {
    let (indexed, states) = parse_pair("", "");
    assert_eq!(reconcile(&indexed, &states), Err(InventoryError::Empty));
}

#[test]
fn non_gitlink_records_do_not_count() {
    let stage = cstr!("100644 {SHA} 0\ttests/_fixtures/_git/README.md\0");
    let indexed = parse_indexed_gitlinks(&stage).expect("index parses");
    assert_eq!(indexed.total(), 0);
    assert_eq!(
        reconcile(&indexed, &BTreeMap::new()),
        Err(InventoryError::Empty)
    );
}

#[test]
fn set_mismatch_is_rejected_in_both_directions() {
    let stage = cstr!(
        "{}{}",
        stage_record("tests/_fixtures/_git/alpha"),
        stage_record("tests/_fixtures/_git/beta")
    );
    let status = cstr!(
        "{}{}",
        status_line(' ', "tests/_fixtures/_git/alpha (v1.0.0)"),
        status_line(' ', "tests/_fixtures/_git/gamma (v1.0.0)")
    );
    let (indexed, states) = parse_pair(&stage, &status);
    assert_eq!(
        reconcile(&indexed, &states),
        Err(InventoryError::SetMismatch {
            index_only: 1,
            status_only: 1,
            first_index_only: "tests/_fixtures/_git/beta".into(),
            first_status_only: "tests/_fixtures/_git/gamma".into(),
        })
    );
}

#[test]
fn invalid_records_are_rejected() {
    assert_eq!(
        parse_indexed_gitlinks("garbage-without-tab\0"),
        Err(InventoryError::InvalidIndexRecord {
            record: "garbage-without-tab".into(),
        })
    );
    assert_eq!(
        parse_submodule_status("not a status line\n"),
        Err(InventoryError::InvalidStatusLine {
            line: "not a status line".into(),
        })
    );
    let truncated = cstr!(" {SHA}\n");
    assert_eq!(
        parse_submodule_status(&truncated),
        Err(InventoryError::InvalidStatusLine {
            line: cstr!(" {SHA}"),
        })
    );
    let duplicate = cstr!(
        "{}{}",
        status_line('-', "tests/_fixtures/_git/alpha"),
        status_line('-', "tests/_fixtures/_git/alpha")
    );
    assert!(matches!(
        parse_submodule_status(&duplicate),
        Err(InventoryError::InvalidStatusLine { .. })
    ));
}

#[test]
fn external_roots_with_canonical_suffix_are_not_canonical() {
    let scratch = tempfile::tempdir().expect("create scratch dir");
    let fake = scratch.path().join(CANONICAL_CORPUS_RELATIVE);
    std::fs::create_dir_all(&fake).expect("create fake corpus root");
    assert!(!is_canonical_root(&fake));
}

#[test]
fn the_checkout_fixture_root_is_canonical() {
    let root = super::workspace_root().join(CANONICAL_CORPUS_RELATIVE);
    assert!(is_canonical_root(&root));
}
