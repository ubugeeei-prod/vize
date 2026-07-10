//! `fmt --check` skips the `<script>` idempotence stabilization pass (the extra
//! oxc parse+format per block) via `FormatOptions::skip_script_stabilization`.
//! These tests pin the behaviour: the change-detection verdict must stay
//! identical to the stabilizing (`--write`) path, including for files that only
//! reach a fixed point after the second pass.

use vize_glyph::{FormatOptions, format_sfc};

fn check_options() -> FormatOptions {
    FormatOptions {
        skip_script_stabilization: true,
        ..FormatOptions::default()
    }
}

#[test]
fn check_mode_matches_write_verdict_for_formatted_and_unformatted() {
    let write = FormatOptions::default();
    let check = check_options();

    let unformatted = "<script setup>\nconst x=1\n</script>\n";
    let canonical = format_sfc(unformatted, &write).unwrap().code;

    // Unformatted source: both paths agree it would change.
    assert!(format_sfc(unformatted, &check).unwrap().changed);
    assert!(format_sfc(unformatted, &write).unwrap().changed);

    // Canonical output: both paths agree it is a no-op.
    assert!(!format_sfc(canonical.as_str(), &check).unwrap().changed);
    assert!(!format_sfc(canonical.as_str(), &write).unwrap().changed);
}

#[test]
fn check_mode_still_flags_script_needing_second_stabilization_pass() {
    // The long union return type below converges only after a second oxc pass
    // (see sfc_script_idempotence.rs). A file holding its once-formatted form is
    // NOT the fixed point `--write` emits, so check must still report it changed
    // even though it skips the stabilization pass.
    let source = r#"<script setup lang="ts">
function getSlotPosition(slotName: string): { style: { left: string, top: string, width: string, height: string }, inPortal: boolean } | null {
  return null
}
</script>
"#;
    let write = FormatOptions::default();
    let check = check_options();

    let once = format_sfc(source, &check).unwrap().code;
    let canonical = format_sfc(source, &write).unwrap().code;
    assert_ne!(
        once, canonical,
        "fixture must actually exercise the non-idempotent script path"
    );

    assert!(
        format_sfc(once.as_str(), &check).unwrap().changed,
        "check must detect a file that is not yet at the stabilized fixed point"
    );
    assert!(
        !format_sfc(canonical.as_str(), &check).unwrap().changed,
        "check must treat the stabilized fixed point as already formatted"
    );
}
