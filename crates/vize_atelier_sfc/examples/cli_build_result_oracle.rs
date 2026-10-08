//! Observe the unchanged SFC API result before the CLI rejects returned errors.

use std::io;

use serde::{Deserialize, Serialize};
use vize_atelier_core::{CodegenOptions, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileExperimentalOptions, SfcCompileOptions, SfcMacroArtifact,
    SfcParseOptions, StyleCompileOptions, TemplateCompileOptions,
    compile_sfc_with_custom_elements_template_syntax_codegen_and_experimental_options, parse_sfc,
};
use vize_l0::{String, ToCompactString};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Input {
    source: String,
    filename: String,
    source_id: String,
}

// Field order and pretty serialization match the original CLI CompileOutput.
#[derive(Serialize)]
struct Output {
    filename: String,
    code: String,
    css: Option<String>,
    errors: Vec<String>,
    warnings: Vec<String>,
    script_lang: String,
    macro_artifacts: Vec<SfcMacroArtifact>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input: Input = serde_json::from_reader(io::stdin().lock())?;
    let descriptor = parse_sfc(
        &input.source,
        SfcParseOptions {
            filename: input.filename.clone(),
            ..Default::default()
        },
    )
    .map_err(|error| io::Error::other(error.message.as_str().to_owned()))?;
    let script_lang = descriptor
        .script_setup
        .as_ref()
        .and_then(|script| script.lang.as_deref())
        .or_else(|| {
            descriptor
                .script
                .as_ref()
                .and_then(|script| script.lang.as_deref())
        })
        .unwrap_or("js")
        .to_compact_string();
    let scoped = descriptor.styles.iter().any(|style| style.scoped);
    let options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: input.filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(input.source_id),
            is_ts: false,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(input.filename.clone()),
            scoped,
            ssr: false,
            is_ts: false,
            custom_renderer: false,
            compiler_options: Some(vize_atelier_dom::DomCompilerOptions::default()),
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: input.filename.clone(),
            scoped,
            ..Default::default()
        },
        vapor: false,
        scope_id: None,
    };
    let result = compile_sfc_with_custom_elements_template_syntax_codegen_and_experimental_options(
        &descriptor,
        options,
        Default::default(),
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        SfcCompileExperimentalOptions::default(),
    )
    .map_err(|error| io::Error::other(error.message.as_str().to_owned()))?;
    serde_json::to_writer_pretty(
        io::stdout().lock(),
        &Output {
            filename: input.filename,
            code: result.code,
            css: result.css,
            errors: result
                .errors
                .into_iter()
                .map(|error| error.message)
                .collect(),
            warnings: result
                .warnings
                .into_iter()
                .map(|error| error.message)
                .collect(),
            script_lang,
            macro_artifacts: result.macro_artifacts,
        },
    )?;
    Ok(())
}
