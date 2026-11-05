use super::*;

#[test]
fn exact_integrated_source_reconstructs_the_merge_base() {
    for file in &FILES {
        let current = std::fs::read_to_string(support::repository().join(file.path)).unwrap();
        source_before(file.path, current);
    }
    preamble_before(vize_canon::virtual_ts::SHARED_PREAMBLE_DTS);
    assert_eq!(source_before("unrelated", "same".to_owned()), "same");
}

#[test]
fn missing_duplicate_and_foreign_integrated_bytes_are_refused() {
    for file in &FILES {
        let current = std::fs::read_to_string(support::repository().join(file.path)).unwrap();
        for changed in [
            String::new(),
            format!("{current}{current}"),
            format!("{current}\n"),
        ] {
            assert!(std::panic::catch_unwind(|| source_before(file.path, changed)).is_err());
        }
    }
    let current = vize_canon::virtual_ts::SHARED_PREAMBLE_DTS;
    assert!(std::panic::catch_unwind(|| preamble_before(&format!("{current}\n"))).is_err());
}
