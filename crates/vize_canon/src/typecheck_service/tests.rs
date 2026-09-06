use super::{
    SfcDiagnosticSeverity, TypeCheckServiceOptions, fallback_position_to_sfc, map_position_to_sfc,
};
use crate::virtual_ts::{VirtualTsOutput, VizeMapping};

#[test]
fn test_sfc_diagnostic_severity() {
    assert_eq!(SfcDiagnosticSeverity::Error, SfcDiagnosticSeverity::Error);
    assert_ne!(SfcDiagnosticSeverity::Error, SfcDiagnosticSeverity::Warning);
}

#[test]
fn test_type_check_service_options_default() {
    let opts = TypeCheckServiceOptions::default();
    assert!(opts.project_root.is_none());
    assert!(opts.tsconfig_path.is_none());
    assert!(!opts.check_cross_component);
    assert!(!opts.check_template);
}

#[test]
fn maps_generated_utf16_columns_before_non_bmp_text() {
    let virtual_ts = VirtualTsOutput {
        code: "\u{1F600}missing;\n".into(),
        mappings: vec![VizeMapping {
            gen_range: 4..11,
            src_range: 100..107,
            sub_spans: Vec::new(),
        }],
        semantic_links: Vec::new(),
    };

    assert_eq!(
        map_position_to_sfc(&virtual_ts, 0, 2, 0, 9, 0, 0),
        (100, 107)
    );
}

#[test]
fn invalid_generated_positions_keep_the_fallback_mapping() {
    assert_eq!(fallback_position_to_sfc(1, 2, 1, 7, 40), (122, 127));
}
