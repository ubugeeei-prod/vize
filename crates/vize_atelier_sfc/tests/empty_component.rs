use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn empty_sfc_emits_empty_component() {
    for source in ["", "\n"] {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse empty SFC");
        let result = compile_sfc(&descriptor, SfcCompileOptions::default())
            .expect("empty SFC placeholders should compile as empty components");

        assert!(
            result.errors.is_empty(),
            "empty SFC should not report compile errors: {:?}",
            result.errors
        );
        assert_eq!(result.css, None);
        assert_eq!(
            result.code, "const _sfc_main = {}\nexport default _sfc_main",
            "empty SFC should emit a minimal component module"
        );
    }
}
