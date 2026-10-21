use serde_json::{Value, json};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileExperimentalOptions, SfcCompileOptions, SfcCompileResult,
    SfcError, SfcParseOptions, SfcScriptOutputMode, compile_sfc, compile_sfc_for_adapter,
    compile_sfc_for_adapter_with_experimental_options,
    compile_sfc_with_template_syntax_and_codegen_options, parse_sfc,
};

use super::options;

pub const IDS: [&str; 10] = [
    "compiler/sfc/imported-component-before-prop",
    "compiler/sfc/scoped-css-v-bind",
    "compiler/sfc/dynamic-loop-ref-for",
    "compiler/sfc/computed-component-unref",
    "compiler/sfc/custom-directive-child-patch",
    "compiler/sfc/dual-script-inline-type-only-imports",
    "compiler/sfc/separate-template-authored-render-binding",
    "compiler/sfc/invalid-template-expression-diagnostic",
    "compiler/sfc/counter-source-map",
    "compiler/sfc/props-default-type-diagnostic",
];
const SOURCES: [&str; 10] = [
    include_str!("../../tests/fixtures/fix-history/imported-component-before-prop.input.txt"),
    include_str!("../../tests/fixtures/fix-history/scoped-css-v-bind.input.txt"),
    include_str!("../../tests/fixtures/fix-history/dynamic-loop-ref-for.input.txt"),
    include_str!("../../tests/fixtures/fix-history/computed-component-unref.input.txt"),
    include_str!("../../tests/fixtures/fix-history/custom-directive-child-patch.input.txt"),
    include_str!("../../tests/fixtures/fix-history/dual-script-inline-type-only-imports.input.txt"),
    include_str!(
        "../../tests/fixtures/fix-history/separate-template-authored-render-binding.input.txt"
    ),
    include_str!(
        "../../tests/fixtures/fix-history/invalid-template-expression-diagnostic.input.txt"
    ),
    include_str!("../../tests/fixtures/fix-history/counter-source-map.input.txt"),
    include_str!("../../tests/fixtures/fix-history/props-default-type-diagnostic.input.txt"),
];

#[derive(Debug)]
struct ParseFailure(SfcError);

impl std::fmt::Display for ParseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}

impl std::error::Error for ParseFailure {}

fn complete_result(result: Result<SfcCompileResult, SfcError>) -> Value {
    match result {
        Ok(SfcCompileResult {
            code,
            css,
            map,
            errors,
            warnings,
            bindings,
            macro_artifacts,
        }) => json!({
            "Ok": { "code": code, "css": css, "map": map, "errors": errors,
                "warnings": warnings, "bindings": bindings, "macroArtifacts": macro_artifacts },
        }),
        Err(error) => json!({ "Err": error }),
    }
}

pub fn observe() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    IDS.iter()
        .enumerate()
        .map(|(index, id)| {
            let descriptor_options = SfcParseOptions {
                filename: match index {
                    2 => "CascaderPanel.vue".into(),
                    8 => "/app/src/Counter.vue".into(),
                    _ => "".into(),
                },
                ..SfcParseOptions::default()
            };
            let options = match index {
                0 => SfcCompileOptions {
                    script: ScriptCompileOptions {
                        is_ts: true,
                        ..ScriptCompileOptions::default()
                    },
                    ..SfcCompileOptions::default()
                },
                1 => SfcCompileOptions {
                    scope_id: Some("test".into()),
                    script: ScriptCompileOptions {
                        id: Some("src/Box.vue".into()),
                        ..ScriptCompileOptions::default()
                    },
                    ..SfcCompileOptions::default()
                },
                6 => SfcCompileOptions {
                    parse: SfcParseOptions {
                        filename: "Foo.vue".into(),
                        ..SfcParseOptions::default()
                    },
                    ..SfcCompileOptions::default()
                },
                8 => SfcCompileOptions {
                    parse: descriptor_options.clone(),
                    script: ScriptCompileOptions {
                        id: Some("/app/src/Counter.vue".into()),
                        inline_template: false,
                        ..ScriptCompileOptions::default()
                    },
                    ..SfcCompileOptions::default()
                },
                9 => SfcCompileOptions {
                    script: ScriptCompileOptions {
                        id: Some("test.vue".into()),
                        ..ScriptCompileOptions::default()
                    },
                    ..SfcCompileOptions::default()
                },
                _ => SfcCompileOptions::default(),
            };
            let output_mode = if index == 0 || index == 2 || index == 6 {
                SfcScriptOutputMode::SeparateTemplate
            } else {
                SfcScriptOutputMode::InlineTemplate
            };
            let codegen = CodegenOptions {
                source_map: index == 8,
                ..CodegenOptions::default()
            };
            let actual_options =
                options::snapshot(&descriptor_options, &options, &codegen, output_mode)?;
            let descriptor = parse_sfc(SOURCES[index], descriptor_options).map_err(ParseFailure)?;
            let (entrypoint, result) = match index {
                6 => (
                    "compile_sfc_for_adapter_with_experimental_options",
                    compile_sfc_for_adapter_with_experimental_options(
                        &descriptor,
                        options,
                        TemplateSyntaxMode::Standard,
                        CustomElementMatcher::default(),
                        codegen,
                        output_mode,
                        SfcCompileExperimentalOptions::default(),
                    ),
                ),
                8 => (
                    "compile_sfc_with_template_syntax_and_codegen_options",
                    compile_sfc_with_template_syntax_and_codegen_options(
                        &descriptor,
                        options,
                        TemplateSyntaxMode::Standard,
                        codegen,
                    ),
                ),
                0 | 2 => (
                    "compile_sfc_for_adapter",
                    compile_sfc_for_adapter(
                        &descriptor,
                        options,
                        TemplateSyntaxMode::Standard,
                        CustomElementMatcher::default(),
                        codegen,
                        output_mode,
                    ),
                ),
                _ => ("compile_sfc", compile_sfc(&descriptor, options)),
            };
            Ok(
                json!({ "id": id, "source": SOURCES[index], "entrypoint": entrypoint,
            "options": actual_options, "result": complete_result(result) }),
            )
        })
        .collect()
}
