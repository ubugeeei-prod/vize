use super::*;

fn compile_with_runtime_names(standalone: bool) -> String {
    let mut compilation = Compilation::new();
    super::register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("Runtime.vue", "<template><div /></template>")
        .unwrap();
    let mut options = SfcCompileOptions::default();
    options.script.inline_template = standalone;
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(options, TemplateSyntaxMode::Standard)
            .with_runtime_names("@acme/vue-runtime", "AcmeVue"),
    )
    .unwrap();
    compilation
        .query::<SfcCompileProduct>(source)
        .unwrap()
        .value()
        .code
        .to_string()
}

#[test]
fn atlas_sfc_runtime_names_reach_backend_and_standalone_assembly() {
    let module = compile_with_runtime_names(false);
    assert!(module.contains("@acme/vue-runtime"), "{module}");

    let standalone = compile_with_runtime_names(true);
    assert!(standalone.contains("AcmeVue"), "{standalone}");
    assert!(!standalone.contains("@acme/vue-runtime"), "{standalone}");
}
