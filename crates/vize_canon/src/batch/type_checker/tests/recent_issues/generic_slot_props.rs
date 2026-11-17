//! #13239: inside a generic parent, a generic child's slot props were
//! `unknown` when the child's slot record keys depend on the parent's type
//! parameter (Nuxt UI's `UTable` `DynamicCellSlots`).
//!
//! The parent passes its still-generic `TItem[]`, so the child's slot record
//! is `CellSlots<TItem, keyof TItem>` and the checker defers the payload. The
//! deferred payload's constraint must stay `{ row: { original: TItem } }`.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

macro_rules! reported {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/generic-slot-props-13239/",
            $path
        ))
    };
}

/// The reporter's files pass, as do a plain `Record<string, F>`, an exact key
/// union, Nuxt UI's whole `TableSlots` intersection and both dynamic slot
/// names. The one control reads a member `TItem` lacks, which proves the
/// payload is typed rather than `any`. Before the fix every scope but the plain
/// record one was TS2339 on `row`.
#[test]
fn generic_child_slot_props_are_inferred_inside_a_generic_parent() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "generic-slot-props-13239",
        &[
            ("tsconfig.json", reported!("tsconfig.json.txt")),
            ("src/components.d.ts", reported!("src/components.d.ts.txt")),
            ("src/my-table.vue", reported!("src/my-table.vue.txt")),
            ("src/plain-table.vue", reported!("src/plain-table.vue.txt")),
            ("src/exact-table.vue", reported!("src/exact-table.vue.txt")),
            ("src/ui-table.vue", reported!("src/ui-table.vue.txt")),
            (
                "src/generic-outer.vue",
                reported!("src/generic-outer.vue.txt"),
            ),
            ("src/plain-outer.vue", reported!("src/plain-outer.vue.txt")),
            (
                "src/generic-variants.vue",
                reported!("src/generic-variants.vue.txt"),
            ),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    let Some(snapshot) = snapshot else {
        return;
    };
    let rows: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| (file.as_str(), *code, message.as_str()))
        .collect();
    assert_eq!(
        rows,
        vec![(
            "src/generic-variants.vue",
            Some(2339),
            "21:52:error Property 'missing' does not exist on type 'TItem'."
        )],
        "{snapshot:?}"
    );
}
