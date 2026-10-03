use super::capture_pack;
use vize_l0::{String, cstr};

#[test]
fn five_whole_const_setup_modules_and_maps_match_actual_source_bound_pinned_fixtures()
-> Result<(), String> {
    let pack = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/native_sfc_const_setup_vue_3_5_35.json"
    ))
    .map_err(|error| cstr!("{error}"))?;
    capture_pack(
        pack,
        5,
        "VIZE_NATIVE_SFC_CONST_SETUP_CAPTURE",
        "vize.native-sfc.const-setup-capture",
    )
}
