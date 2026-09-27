//! Complete public observations for original map and diagnostic fix inputs.

use serde_json::{Value, json};
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcError, SfcParseOptions, compile_sfc,
    compile_sfc_with_template_syntax_and_codegen_options, parse_sfc,
    validate_script_setup_semantics_located,
};

use super::fix_history_map_diagnostic_options as fixture_options;

const COUNTER_ID: &str = "counter-source-map";
const DIAGNOSTIC_ID: &str = "props-default-type-diagnostic";
pub const CASE_IDS: &[&str] = &[COUNTER_ID, DIAGNOSTIC_ID];
const COUNTER: &str = include_str!("../fixtures/fix-history/counter-source-map.input.txt");
const DIAGNOSTIC: &str =
    include_str!("../fixtures/fix-history/props-default-type-diagnostic.input.txt");

#[derive(Debug)]
struct ParseFailure(SfcError);

impl std::fmt::Display for ParseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, formatter)
    }
}

impl std::error::Error for ParseFailure {}

pub fn observe() -> Result<Value, Box<dyn std::error::Error>> {
    let parse = SfcParseOptions {
        filename: "/app/src/Counter.vue".into(),
        ..SfcParseOptions::default()
    };
    let options = SfcCompileOptions {
        parse: parse.clone(),
        script: ScriptCompileOptions {
            id: Some("/app/src/Counter.vue".into()),
            inline_template: false,
            ..ScriptCompileOptions::default()
        },
        ..SfcCompileOptions::default()
    };
    let codegen = CodegenOptions {
        source_map: true,
        ..CodegenOptions::default()
    };
    let map_options =
        fixture_options::observe(&parse, &options, &codegen, &CustomElementMatcher::default())?;
    let descriptor = parse_sfc(COUNTER, parse).map_err(ParseFailure)?;
    let map_result = compile_sfc_with_template_syntax_and_codegen_options(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        codegen,
    );

    let parse = SfcParseOptions::default();
    let options = SfcCompileOptions {
        script: ScriptCompileOptions {
            id: Some("test.vue".into()),
            ..ScriptCompileOptions::default()
        },
        ..SfcCompileOptions::default()
    };
    let diagnostic_options = fixture_options::observe(
        &parse,
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
    )?;
    let descriptor = parse_sfc(DIAGNOSTIC, parse).map_err(ParseFailure)?;
    let setup = descriptor
        .script_setup
        .as_ref()
        .ok_or("authored diagnostic source lost its script setup block")?;
    let located_result =
        validate_script_setup_semantics_located(&setup.content, setup.loc.start, DIAGNOSTIC);
    let located_context = json!({
        "entrypoint": "validate_script_setup_semantics_located",
        "scriptSetupContent": setup.content, "blockStart": setup.loc.start,
        "sfcSource": DIAGNOSTIC,
    });
    let diagnostic_result = compile_sfc(&descriptor, options);
    Ok(json!({
        "schema": "vize.sfc.map-diagnostic-history-observation", "version": 1,
        "cases": [
            {
                "id": COUNTER_ID,
                "entrypoint": "compile_sfc_with_template_syntax_and_codegen_options",
                "options": map_options, "compileResult": serde_json::to_value(map_result)?,
            },
            {
                "id": DIAGNOSTIC_ID, "entrypoint": "compile_sfc",
                "options": diagnostic_options,
                "compileResult": serde_json::to_value(diagnostic_result)?,
                "locatedContext": located_context,
                "locatedResult": serde_json::to_value(located_result)?,
            },
        ],
    }))
}
