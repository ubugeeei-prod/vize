//! Pinned DOM modules for the two output changes of the normal-script route
//! (#6844): ambient `declare` names bind nothing, and a named `default`
//! import from a `.vue` file is the component itself. Both follow Vue's
//! `compileScript`. `VIZE_ORDINARY_FIXTURE_WRITE=1` rewrites the pins.

const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/compiler/"
);

fn pinned(code: &str, expected: &str, case: &str) {
    if std::env::var_os("VIZE_ORDINARY_FIXTURE_WRITE").is_some() {
        let path = std::path::Path::new(ROOT)
            .join(case)
            .join("module.expected.txt");
        std::fs::write(path, code).expect("write");
        return;
    }
    assert_eq!(code, expected, "{case}");
}

fn dom_module(source: &str, filename: &str) -> vize_l0::String {
    let descriptor = crate::parse_sfc(
        source,
        crate::SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .expect("SFC");
    let result = crate::compile_sfc(
        &descriptor,
        crate::SfcCompileOptions {
            parse: crate::SfcParseOptions {
                filename: filename.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile");
    assert_eq!(result.errors.len(), 0);
    result.code
}

#[test]
fn ambient_declarations_in_a_normal_script_bind_nothing() {
    let code = dom_module(
        include_str!(
            "../../../../../../tests/_fixtures/differential/compiler/normal-script-ambient-declare-6844/Ambient.vue.txt"
        ),
        "Ambient.vue",
    );
    pinned(
        code.as_str(),
        include_str!(
            "../../../../../../tests/_fixtures/differential/compiler/normal-script-ambient-declare-6844/module.expected.txt"
        ),
        "normal-script-ambient-declare-6844",
    );
}

#[test]
fn a_named_default_vue_import_is_the_component() {
    let code = dom_module(
        include_str!(
            "../../../../../../tests/_fixtures/differential/compiler/named-default-vue-import-6844/Named.vue.txt"
        ),
        "Named.vue",
    );
    pinned(
        code.as_str(),
        include_str!(
            "../../../../../../tests/_fixtures/differential/compiler/named-default-vue-import-6844/module.expected.txt"
        ),
        "named-default-vue-import-6844",
    );
}
