//! Complete licensed-original compiler receipts for the n8n before/after gate.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "forensic example serializes complete source and output"
)]

use serde_json::{Value, json};
use std::{env, error::Error, fs, path::Path};
use vize_atelier_core::{
    CodegenOptions, TemplateSyntaxMode,
    options::{CodegenMode, CustomElementMatcher},
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_legacy_with_options};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    StyleCompileOptions, TemplateCompileOptions, compile_sfc_for_adapter, parse_sfc,
};
mod native;

use vize_l0::Allocator;
use vize_l1_to_l2::{DomEmitMode, DomEmitOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let [fixture_root, output_root] = args.as_slice() else {
        return Err("usage: n8n_compiler_custody <n8n-root> <output-root>".into());
    };
    let cases: Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/differential/compiler/n8n-adoption/cases.json"
    ))?;
    let paths = cases["cases"].as_array().ok_or("missing cases")?;
    fs::create_dir_all(output_root)?;
    for license in ["LICENSE.md", "LICENSE_EE.md"] {
        fs::copy(
            Path::new(fixture_root).join(license),
            Path::new(output_root).join(license),
        )?;
    }
    for case in paths {
        let path = case["path"].as_str().ok_or("missing case path")?;
        let source = fs::read_to_string(Path::new(fixture_root).join(path))?;
        let descriptor = parse_sfc(&source, SfcParseOptions::default())
            .map_err(|error| format!("original SFC parse failed: {error:?}"))?;
        let template = descriptor
            .template
            .as_ref()
            .ok_or("missing original template")?;
        let is_ts = descriptor
            .script
            .iter()
            .chain(descriptor.script_setup.iter())
            .any(|block| matches!(block.lang.as_deref(), Some("ts" | "tsx")));
        let mut rows = Vec::new();
        for prefixed in [false, true] {
            let allocator = Allocator::new();
            let (_, errors, legacy) = compile_template_legacy_with_options(
                &allocator,
                &template.content,
                DomCompilerOptions {
                    is_ts,
                    prefix_identifiers: prefixed,
                    mode: if prefixed {
                        CodegenMode::Module
                    } else {
                        CodegenMode::Function
                    },
                    ..Default::default()
                },
            );
            let native = native::capture(
                &template.content,
                &DomEmitOptions {
                    is_ts,
                    prefix_identifiers: prefixed,
                    mode: if prefixed {
                        DomEmitMode::Module
                    } else {
                        DomEmitMode::Function
                    },
                    ..DomEmitOptions::DEFAULT
                },
            );
            rows.push(json!({
                "kind": "template", "prefixIdentifiers": prefixed, "isTs": is_ts,
                "legacy": {"preamble": legacy.preamble, "code": legacy.code, "assembled": format!("{}\n{}", legacy.preamble, legacy.code),
                    "errors": errors.iter().map(|e| json!({"code": format!("{:?}", e.code), "message": e.message, "location": e.loc})).collect::<Vec<_>>()},
                "native": native
            }));
        }
        for inline in [false, true] {
            for source_map in [false, true] {
                let filename = vize_l0::String::from(path);
                let scoped = descriptor.styles.iter().any(|style| style.scoped);
                let options = SfcCompileOptions {
                    parse: SfcParseOptions {
                        filename: filename.clone(),
                        ..Default::default()
                    },
                    script: ScriptCompileOptions {
                        id: Some(filename.clone()),
                        ..Default::default()
                    },
                    template: TemplateCompileOptions {
                        id: Some(filename.clone()),
                        scoped,
                        compiler_options: Some(DomCompilerOptions::default()),
                        ..Default::default()
                    },
                    style: StyleCompileOptions {
                        id: filename,
                        scoped,
                        source_map,
                        ..Default::default()
                    },
                    vapor: false,
                    scope_id: None,
                };
                let output_mode = if inline {
                    SfcScriptOutputMode::InlineTemplate
                } else {
                    SfcScriptOutputMode::SeparateTemplate
                };
                let compile = || {
                    compile_sfc_for_adapter(
                        &descriptor,
                        options.clone(),
                        TemplateSyntaxMode::Standard,
                        CustomElementMatcher::default(),
                        CodegenOptions {
                            source_map,
                            ..Default::default()
                        },
                        output_mode,
                    )
                };
                let selected = compile();
                let legacy = vize_atelier_dom::differential::with_legacy_lane(compile);
                rows.push(json!({"kind": "sfc", "inline": inline, "sourceMap": source_map, "selected": selected, "legacy": legacy}));
            }
        }
        let filename = Path::new(path)
            .file_name()
            .ok_or("missing filename")?
            .to_string_lossy();
        fs::write(
            Path::new(output_root).join(format!("{filename}.json")),
            serde_json::to_vec_pretty(
                &json!({"path": path, "originalSource": source, "rows": rows}),
            )?,
        )?;
    }
    let source = include_str!(
        "../../../../tests/_fixtures/differential/compiler/n8n-default-slot-loop/template.vue.txt"
    );
    let allocator = Allocator::new();
    let (_, errors, legacy) =
        compile_template_legacy_with_options(&allocator, source, DomCompilerOptions::default());
    let native = native::capture(source, &DomEmitOptions::DEFAULT);
    fs::write(
        Path::new(output_root).join("n8n-default-slot-loop.json"),
        serde_json::to_vec_pretty(&json!({
            "path": "tests/_fixtures/differential/compiler/n8n-default-slot-loop/template.vue.txt",
            "originalSource": source,
            "rows": [{"kind": "template", "prefixIdentifiers": false, "isTs": false,
                "legacy": {"preamble": legacy.preamble, "code": legacy.code,
                    "assembled": format!("{}\n{}", legacy.preamble, legacy.code),
                    "errors": errors.iter().map(|e| json!({"code": format!("{:?}", e.code),
                        "message": e.message, "location": e.loc})).collect::<Vec<_>>()},
                "native": native}]
        }))?,
    )?;
    let diagnostic_contract: Value = serde_json::from_str(include_str!(
        "../../../../tests/_fixtures/differential/compiler/n8n-if-key-regression/diagnostics.json"
    ))?;
    fs::write(
        Path::new(output_root).join("diagnostic-contract.json"),
        serde_json::to_vec_pretty(&native::expected_contract(&diagnostic_contract)?)?,
    )?;
    fs::write(
        Path::new(output_root).join("cases.json"),
        serde_json::to_vec_pretty(&cases)?,
    )?;
    Ok(())
}
