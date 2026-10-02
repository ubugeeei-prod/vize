use serde_json::{Value, json};
use vize_atelier_core::{CodegenOptions, options::CustomElementMatcher};
use vize_atelier_sfc::{
    PadOption, PropsDestructure, ScriptCompileOptions, SfcCompileExperimentalOptions,
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, StyleCompileOptions,
    TemplateCompileOptions,
};

fn parse(options: &SfcParseOptions) -> Result<Value, &'static str> {
    let SfcParseOptions {
        filename,
        source_map,
        pad,
        ignore_empty,
        template_parse_options,
    } = options;
    if template_parse_options.is_some() {
        return Err("custom template parser is not supported");
    }
    Ok(json!({ "filename": filename, "source_map": source_map,
        "pad": match pad { PadOption::None => "None", PadOption::Line => "Line", PadOption::Space => "Space" },
        "ignore_empty": ignore_empty, "template_parse_options": null }))
}

fn codegen(options: &CodegenOptions) -> Value {
    let CodegenOptions {
        mode,
        prefix_identifiers,
        source_map,
        filename,
        component_name,
        scope_id,
        ssr,
        optimize_imports,
        runtime_module_name,
        runtime_global_name,
        is_ts,
        inline,
        binding_metadata,
        cache_handlers,
    } = options;
    json!({ "mode": match mode { vize_atelier_core::options::CodegenMode::Function => "Function",
        vize_atelier_core::options::CodegenMode::Module => "Module" },
        "prefix_identifiers": prefix_identifiers, "source_map": source_map, "filename": filename,
        "component_name": component_name, "scope_id": scope_id, "ssr": ssr,
        "optimize_imports": optimize_imports, "runtime_module_name": runtime_module_name,
        "runtime_global_name": runtime_global_name, "is_ts": is_ts, "inline": inline,
        "binding_metadata": binding_metadata, "cache_handlers": cache_handlers })
}

pub fn snapshot(
    descriptor: &SfcParseOptions,
    options: &SfcCompileOptions,
    mode: SfcScriptOutputMode,
) -> Result<Value, Box<dyn std::error::Error>> {
    let SfcCompileOptions {
        parse: compile_parse,
        script,
        template,
        style,
        vapor,
        scope_id,
    } = options;
    let ScriptCompileOptions {
        id: script_id,
        inline_template,
        is_ts: script_ts,
        reactive_props_destructure,
        props_destructure,
        define_model,
    } = script;
    let TemplateCompileOptions {
        id: template_id,
        ssr,
        ssr_css_vars,
        scoped: template_scoped,
        is_prod,
        is_ts: template_ts,
        custom_renderer,
        dialect,
        compiler_options,
    } = template;
    let StyleCompileOptions {
        id: style_id,
        scoped: style_scoped,
        trim,
        source_map,
        preprocessor_lang,
        data_attrs,
    } = style;
    let custom_elements = CustomElementMatcher::default();
    if !custom_elements.is_empty()
        || compiler_options.is_some()
        || *dialect != vize_l0::config::VueVersion::V3
    {
        return Err("original default matcher and Vue 3 context changed".into());
    }
    let SfcCompileExperimentalOptions { self_component } = SfcCompileExperimentalOptions::default();
    Ok(json!({ "descriptorParseOptions": parse(descriptor)?,
        "compileOptions": { "parse": parse(compile_parse)?,
            "script": { "id": script_id, "inline_template": inline_template, "is_ts": script_ts,
                "reactive_props_destructure": reactive_props_destructure,
                "props_destructure": match props_destructure { PropsDestructure::False => "False",
                    PropsDestructure::True => "True", PropsDestructure::Error => "Error" }, "define_model": define_model },
            "template": { "id": template_id, "ssr": ssr, "ssr_css_vars": ssr_css_vars,
                "scoped": template_scoped, "is_prod": is_prod, "is_ts": template_ts,
                "custom_renderer": custom_renderer, "dialect": "V3", "compiler_options": null },
            "style": { "id": style_id, "scoped": style_scoped, "trim": trim, "source_map": source_map,
                "preprocessor_lang": preprocessor_lang, "data_attrs": data_attrs },
            "vapor": vapor, "scope_id": scope_id },
        "effectiveAdapterInputs": { "templateSyntax": "Standard",
            "customElements": { "construction": "CustomElementMatcher::default()" },
            "codegen": codegen(&CodegenOptions::default()),
            "scriptOutput": match mode { SfcScriptOutputMode::SeparateTemplate => "SeparateTemplate",
                SfcScriptOutputMode::InlineTemplate => "InlineTemplate" },
            "experimental": { "self_component": self_component } }
    }))
}
