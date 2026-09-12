//! TS-17 S2 to S3 pass snapshots.
//!
//! These are full normalized S3 Folio snapshots plus the exported partition
//! facts. Targeted structural tests can miss a stale pass decision; the
//! snapshot is the oracle for the whole lowered artifact.

use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, String, appendln};
use vize_s2_to_s3::{Lowered, lower};
use vize_s3::folio::S3Folio;
use vize_s3::verify::verify;

macro_rules! assert_s3_snapshot {
    ($value:expr) => {{
        #[allow(clippy::disallowed_macros)]
        {
            ::insta::assert_snapshot!($value);
        }
    }};
}

fn snapshot(source: &str) -> String {
    let allocator = Allocator::default();
    let (tree, errors) = vize_s1::parse(&allocator, source);
    let s2 = vize_s1_to_s2::lower(&allocator, &tree, &errors);
    let lowered = lower(&allocator, &s2.root);
    assert_eq!(verify(&lowered.program), []);

    let mut output = S3Folio::of(&lowered.program).print_to_string(FolioMode::Full);
    output.push_str(partition_page(&lowered).as_str());
    output
}

fn partition_page(lowered: &Lowered<'_>) -> String {
    let mut output = String::from("[s3-partition-facts]\n");
    for fact in lowered.partition.ops.iter() {
        appendln!(
            output,
            "op=",
            @fact.op.index(),
            " kind=",
            fact.kind.as_str(),
            " span=",
            @fact.span.start,
            #':',
            @fact.span.end
        );
    }
    output.push('\n');
    output
}

#[test]
fn static_and_dynamic_template_snapshots_s3_folio() {
    assert_s3_snapshot!(snapshot(
        r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#
    ));
}

#[test]
fn control_and_slots_template_snapshots_s3_folio() {
    assert_s3_snapshot!(snapshot(
        r#"<section><p v-if="ready">ready</p><slot name="body"><span v-text="fallback" /></slot></section>"#
    ));
}
