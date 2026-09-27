//! Read actual option values; refuse default drift before observing output.

use serde_json::{Value, json};
use vize_atelier_core::{CodegenOptions, options::CustomElementMatcher};
use vize_atelier_sfc::{PadOption, PropsDestructure, SfcCompileOptions, SfcParseOptions};

fn parse(options: &SfcParseOptions) -> Result<Value, &'static str> {
    if options.template_parse_options.is_some() {
        return Err("profile does not support a custom template parser");
    }
    Ok(json!({
        "filename": options.filename,
        "source_map": options.source_map,
        "pad": match options.pad {
            PadOption::None => "None", PadOption::Line => "Line", PadOption::Space => "Space",
        },
        "ignore_empty": options.ignore_empty,
        "template_parse_options": null,
    }))
}

pub fn validate(
    pinned: &Value,
    descriptor: &SfcParseOptions,
    options: &SfcCompileOptions,
    codegen: &CodegenOptions,
    custom_elements: &CustomElementMatcher,
) -> Result<(), Box<dyn std::error::Error>> {
    if !custom_elements.is_empty()
        || options.template.compiler_options.is_some()
        || options.template.dialect != vize_l0::config::VueVersion::V3
    {
        return Err("profile defaults gained a custom matcher or DOM options".into());
    }
    let actual_compile = json!({
        "parse": parse(&options.parse)?,
        "script": {
            "id": options.script.id, "inline_template": options.script.inline_template,
            "is_ts": options.script.is_ts,
            "reactive_props_destructure": options.script.reactive_props_destructure,
            "props_destructure": match options.script.props_destructure {
                PropsDestructure::False => "False", PropsDestructure::True => "True",
                PropsDestructure::Error => "Error",
            },
            "define_model": options.script.define_model,
        },
        "template": {
            "id": options.template.id, "ssr": options.template.ssr,
            "ssr_css_vars": options.template.ssr_css_vars, "scoped": options.template.scoped,
            "is_prod": options.template.is_prod, "is_ts": options.template.is_ts,
            "custom_renderer": options.template.custom_renderer,
            "dialect": "V3", "compiler_options": null,
        },
        "style": {
            "id": options.style.id, "scoped": options.style.scoped, "trim": options.style.trim,
            "source_map": options.style.source_map,
            "preprocessor_lang": options.style.preprocessor_lang,
            "data_attrs": options.style.data_attrs,
        },
        "vapor": options.vapor, "scope_id": options.scope_id,
    });
    let actual_codegen = json!({
        "mode": match codegen.mode {
            vize_atelier_core::options::CodegenMode::Function => "Function",
            vize_atelier_core::options::CodegenMode::Module => "Module",
        },
        "prefix_identifiers": codegen.prefix_identifiers, "source_map": codegen.source_map,
        "filename": codegen.filename, "component_name": codegen.component_name,
        "scope_id": codegen.scope_id, "ssr": codegen.ssr,
        "optimize_imports": codegen.optimize_imports,
        "runtime_module_name": codegen.runtime_module_name,
        "runtime_global_name": codegen.runtime_global_name,
        "is_ts": codegen.is_ts, "inline": codegen.inline,
        "binding_metadata": codegen.binding_metadata, "cache_handlers": codegen.cache_handlers,
    });
    if pinned.get("descriptorParseOptions") != Some(&parse(descriptor)?)
        || pinned.get("compileOptions") != Some(&actual_compile)
        || pinned
            .get("effectiveAdapterInputs")
            .and_then(|inputs| inputs.get("codegen"))
            != Some(&actual_codegen)
    {
        return Err("actual compiler defaults differ from the authored fixture profile".into());
    }
    Ok(())
}
