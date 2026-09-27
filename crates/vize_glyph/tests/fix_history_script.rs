//! Complete output bytes for historical script stabilization regressions.
//! Binary snapshots retain final newlines and CR/LF without normalization.

use vize_glyph::{FormatOptions, format_script, format_sfc};

fn snapshot_bytes(name: &str, output: &str) {
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_binary_snapshot!(name, output.as_bytes().to_vec());
    });
}

const SIGNATURE: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter-history/script-signature/input.ts"
);
const ZOD: &str =
    include_str!("../../../tests/_fixtures/differential/formatter-history/script-zod/input.ts");
const SFC_SIGNATURE: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter-history/sfc-signature/input.vue.txt"
);
const SFC_ZOD: &str =
    include_str!("../../../tests/_fixtures/differential/formatter-history/sfc-zod/input.vue.txt");

#[test]
fn script_signature_reaches_the_pinned_fixed_point() {
    let options = FormatOptions::default();
    let first = format_script(SIGNATURE, &options).unwrap();
    snapshot_bytes("history_script_signature.txt", &first);
    let second = format_script(&first, &options).unwrap();
    let third = format_script(&second, &options).unwrap();
    assert_eq!(first, second);
    assert_eq!(second, third);
}

#[test]
fn script_zod_reaches_the_pinned_fixed_point() {
    let options = FormatOptions::default();
    let first = format_script(ZOD, &options).unwrap();
    snapshot_bytes("history_script_zod.txt", &first);
    let second = format_script(&first, &options).unwrap();
    let third = format_script(&second, &options).unwrap();
    assert_eq!(first, second);
    assert_eq!(second, third);
}

#[test]
fn sfc_signature_reaches_the_pinned_fixed_point() {
    let options = FormatOptions::default();
    let first = format_sfc(SFC_SIGNATURE, &options).unwrap();
    snapshot_bytes("history_sfc_signature.txt", &first.code);
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();
    assert_eq!(first.code, second.code);
    assert_eq!(second.code, third.code);
    assert!(!second.changed);
    assert!(!third.changed);
}

#[test]
fn sfc_zod_reaches_the_pinned_fixed_point() {
    let options = FormatOptions::default();
    let first = format_sfc(SFC_ZOD, &options).unwrap();
    snapshot_bytes("history_sfc_zod.txt", &first.code);
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();
    assert_eq!(first.code, second.code);
    assert_eq!(second.code, third.code);
    assert!(!second.changed);
    assert!(!third.changed);
}

#[test]
fn script_check_mode_keeps_the_pinned_single_pass_output() {
    let check = FormatOptions {
        skip_script_stabilization: true,
        ..FormatOptions::default()
    };
    let once = format_script(SIGNATURE, &check).unwrap();
    snapshot_bytes("history_script_signature_check.txt", &once);
    let canonical = format_script(SIGNATURE, &FormatOptions::default()).unwrap();
    assert_ne!(once, canonical);
    assert_ne!(format_script(&once, &check).unwrap(), once);
    assert_eq!(format_script(&canonical, &check).unwrap(), canonical);
}

#[test]
fn sfc_check_mode_keeps_output_and_change_detection() {
    let check = FormatOptions {
        skip_script_stabilization: true,
        ..FormatOptions::default()
    };
    let once = format_sfc(SFC_SIGNATURE, &check).unwrap();
    snapshot_bytes("history_sfc_signature_check.txt", &once.code);
    let canonical = format_sfc(SFC_SIGNATURE, &FormatOptions::default()).unwrap();
    assert_ne!(once.code, canonical.code);
    assert!(once.changed);
    assert!(canonical.changed);
    assert!(format_sfc(&once.code, &check).unwrap().changed);
    assert!(!format_sfc(&canonical.code, &check).unwrap().changed);
}
