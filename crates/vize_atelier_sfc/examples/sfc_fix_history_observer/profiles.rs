use serde_json::{Value, json};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcCompileResult, SfcError, SfcParseOptions,
    SfcScriptOutputMode, compile_sfc, compile_sfc_for_adapter, parse_sfc,
};

use super::options;

pub const IDS: [&str; 5] = [
    "compiler/sfc/imported-component-before-prop",
    "compiler/sfc/scoped-css-v-bind",
    "compiler/sfc/dynamic-loop-ref-for",
    "compiler/sfc/computed-component-unref",
    "compiler/sfc/custom-directive-child-patch",
];
const SOURCES: [&str; 5] = [
    include_str!("../../tests/fixtures/fix-history/imported-component-before-prop.input.txt"),
    include_str!("../../tests/fixtures/fix-history/scoped-css-v-bind.input.txt"),
    include_str!("../../tests/fixtures/fix-history/dynamic-loop-ref-for.input.txt"),
    include_str!("../../tests/fixtures/fix-history/computed-component-unref.input.txt"),
    include_str!("../../tests/fixtures/fix-history/custom-directive-child-patch.input.txt"),
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
                filename: if index == 2 {
                    "CascaderPanel.vue".into()
                } else {
                    "".into()
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
                _ => SfcCompileOptions::default(),
            };
            let output_mode = if index == 0 || index == 2 {
                SfcScriptOutputMode::SeparateTemplate
            } else {
                SfcScriptOutputMode::InlineTemplate
            };
            let actual_options = options::snapshot(&descriptor_options, &options, output_mode)?;
            let descriptor = parse_sfc(SOURCES[index], descriptor_options).map_err(ParseFailure)?;
            let (entrypoint, result) = match output_mode {
                SfcScriptOutputMode::SeparateTemplate => (
                    "compile_sfc_for_adapter",
                    compile_sfc_for_adapter(
                        &descriptor,
                        options,
                        TemplateSyntaxMode::Standard,
                        CustomElementMatcher::default(),
                        CodegenOptions::default(),
                        output_mode,
                    ),
                ),
                SfcScriptOutputMode::InlineTemplate => {
                    ("compile_sfc", compile_sfc(&descriptor, options))
                }
            };
            Ok(
                json!({ "id": id, "source": SOURCES[index], "entrypoint": entrypoint,
            "options": actual_options, "result": complete_result(result) }),
            )
        })
        .collect()
}
