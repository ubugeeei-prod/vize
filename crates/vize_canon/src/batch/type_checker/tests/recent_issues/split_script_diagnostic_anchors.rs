//! Source anchors for diagnostics in a split `<script>` / `<script setup>` SFC (#3756).
//!
//! Croquis analyzes the blocks in one synthetic buffer separated by a single
//! newline. Source mappings must account for the authored closing and opening
//! tags between them instead of treating that synthetic buffer as contiguous.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::String;

/// `vue-tsc 3.3.4` with TypeScript 6.0.3, on this byte-identical fixture:
///
/// ```text
/// src/App.vue(10,43): error TS7006: Parameter 'it' implicitly has an 'any' type.
/// ```
///
/// Before the split-script mapping was rebased at the setup block, Vize placed
/// the same diagnostic at line 10, column 9: the byte gap containing
/// `</script>` and `<script setup>` was subtracted from the parameter column.
#[test]
fn split_script_setup_diagnostic_matches_vue_tsc_position() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "split-script-setup-diagnostic-anchor",
        &[(
            "src/App.vue",
            r#"<script lang="ts">
export type SearchQuery = { value: string };
</script>

<script setup lang="ts">
const customEmojis: any = {};
const gridItems: any = {};

function refreshGridItems() {
	gridItems.value = customEmojis.value.map(it => ({
		id: it.id,
	}));
}
</script>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/App.vue"),
            Some(7006),
            String::from("10:43:error Parameter 'it' implicitly has an 'any' type."),
        )],
    );
}
