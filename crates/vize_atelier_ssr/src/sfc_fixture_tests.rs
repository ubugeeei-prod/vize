//! The pinned `compiler/sfc/ssr-declared-directives` and
//! `compiler/sfc/ssr-setup-dynamic-attributes` differential cases: the whole
//! SSR module the public SFC API emits for each authored `Layout.vue`.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

fn fixture(name: &str, file: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler")
        .join(name)
        .join(file)
}

#[test]
fn setup_asset_fixtures_compile_to_their_pinned_modules() {
    let bless = std::env::var_os("VIZE_BLESS_SSR_SETUP_ASSET_FIXTURES").is_some();
    for name in ["ssr-declared-directives", "ssr-setup-dynamic-attributes"] {
        let source = std::fs::read_to_string(fixture(name, "Layout.vue.txt")).expect("input");
        let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("SFC");
        let result = compile_sfc(
            &descriptor,
            SfcCompileOptions {
                parse: SfcParseOptions {
                    filename: "Layout.vue".into(),
                    ..Default::default()
                },
                template: TemplateCompileOptions {
                    ssr: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("compile");
        assert_eq!(result.errors.len(), 0, "{name}: {:?}", result.errors);
        let path = fixture(name, "module.expected.txt");
        if bless {
            std::fs::write(&path, result.code.as_str()).expect("bless module");
            continue;
        }
        let expected = std::fs::read_to_string(&path).expect("pinned module");
        assert_eq!(result.code.as_str(), expected, "{name}");
    }
}
