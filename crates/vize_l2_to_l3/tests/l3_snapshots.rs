//! TS-17 L2 to L3 pass snapshots.
//!
//! These are full normalized L3 Folio snapshots plus the exported partition
//! facts. Targeted structural tests can miss a stale pass decision; the
//! snapshot is the oracle for the whole lowered artifact.

use vize_davinci::folio::{Folio, FolioMode};
use vize_l0::{Allocator, String};
use vize_l2_to_l3::{L3PartitionFolio, lower};
use vize_l3::folio::L3Folio;
use vize_l3::verify::verify;

macro_rules! assert_l3_snapshot {
    ($value:expr) => {{
        #[allow(clippy::disallowed_macros)]
        {
            ::insta::assert_snapshot!($value);
        }
    }};
}

fn snapshot(source: &str) -> String {
    let allocator = Allocator::default();
    let (tree, errors) = vize_l1::parse(&allocator, source);
    let s2 = vize_l1_to_l2::lower(&allocator, &tree, &errors);
    let lowered = lower(&allocator, &s2.root);
    assert_eq!(verify(&lowered.program), []);

    let mut output = L3Folio::of(&lowered.program).print_to_string(FolioMode::Full);
    output.push_str(
        L3PartitionFolio::of(&lowered.partition)
            .print_to_string(FolioMode::Full)
            .as_str(),
    );
    output
}

#[test]
fn static_and_dynamic_template_snapshots_l3_folio() {
    assert_l3_snapshot!(snapshot(
        r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#
    ));
}

#[test]
fn control_and_slots_template_snapshots_l3_folio() {
    assert_l3_snapshot!(snapshot(
        r#"<section><p v-if="ready">ready</p><slot name="body"><span v-text="fallback" /></slot></section>"#
    ));
}
