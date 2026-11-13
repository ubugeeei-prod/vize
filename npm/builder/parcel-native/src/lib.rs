use parcel_plugin::{Asset, BuildMode, Diagnostic, Options, Plugin, register_plugin};
use serde::Deserialize;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    StyleCompileOptions, TemplateCompileOptions, collect_template_asset_urls,
    compile_sfc_for_adapter, generate_bundler_scope_id,
    module_shape::finalize_module_output_with_map, parse_sfc,
};
use vize_carton::{String, cstr};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
enum StyleMode {
    #[default]
    Inject,
    Ignore,
    Error,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct TransformerConfig {
    ssr: bool,
    vapor: bool,
    styles: StyleMode,
    is_production: Option<bool>,
    transform_asset_urls: bool,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self {
            ssr: false,
            vapor: false,
            styles: StyleMode::Inject,
            is_production: None,
            transform_asset_urls: true,
        }
    }
}

struct VizeTransformer {
    config: TransformerConfig,
}

impl Plugin for VizeTransformer {
    fn new(config: &[u8]) -> Result<Self, Diagnostic> {
        let config = parse_transformer_config(config)
            .map_err(|message| Diagnostic::new(message.into_string()))?;
        Ok(Self { config })
    }

    fn transform(&self, asset: &mut Asset, options: &Options) -> Result<(), Diagnostic> {
        let source = asset.content()?;
        let file_path = asset.file_path();
        let filename = file_path.to_string_lossy();
        let project_root = options.project_root();
        let project_root = project_root.to_string_lossy();
        let is_production = self
            .config
            .is_production
            .unwrap_or_else(|| matches!(options.mode(), BuildMode::Production));
        let output = compile_source(
            &source,
            &filename,
            &project_root,
            is_production,
            &self.config,
        )
        .map_err(|message| Diagnostic::new(message.into_string()).with_file(&file_path))?;

        for warning in output.warnings {
            options.log_diagnostic(
                &Diagnostic::warning(warning.into_string()).with_file(file_path.clone()),
            );
        }
        match output.map {
            Some(map) => asset.set_content_with_map(output.code, map),
            None => asset.set_content(output.code),
        }
        asset.set_type("js");
        Ok(())
    }
}

fn parse_transformer_config(config: &[u8]) -> Result<TransformerConfig, String> {
    if config.is_empty() {
        return Ok(TransformerConfig::default());
    }
    let value = serde_json::from_slice::<serde_json::Value>(config)
        .map_err(|error| cstr!("Invalid Vize transformer config: {error}"))?;
    if !value.is_object() {
        return Err(cstr!(
            "Invalid Vize transformer config: expected a JSON object"
        ));
    }
    serde_json::from_value(value).map_err(|error| cstr!("Invalid Vize transformer config: {error}"))
}

register_plugin!(VizeTransformer);

#[derive(Debug)]
struct CompileOutput {
    code: String,
    map: Option<String>,
    warnings: Vec<String>,
}

fn compile_source(
    source: &str,
    filename: &str,
    project_root: &str,
    is_production: bool,
    config: &TransformerConfig,
) -> Result<CompileOutput, String> {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            source_map: true,
            ..Default::default()
        },
    )
    .map_err(|error| error.message)?;
    validate_descriptor(&descriptor, filename)?;

    let has_scoped = descriptor.styles.iter().any(|style| style.scoped);
    let scope_id = generate_bundler_scope_id(
        filename,
        Some(project_root),
        is_production,
        is_production.then_some(source),
    );
    let mut dom_options = vize_atelier_dom::DomCompilerOptions::default();
    if has_scoped {
        dom_options.scope_id = Some(cstr!("data-v-{scope_id}"));
    }
    let compile_options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.into(),
            source_map: true,
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.into()),
            inline_template: is_production && !config.ssr,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            scoped: has_scoped,
            ssr: config.ssr,
            is_prod: is_production,
            compiler_options: Some(dom_options),
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.into(),
            scoped: has_scoped,
            ..Default::default()
        },
        vapor: config.vapor,
        scope_id: Some(scope_id.clone()),
    };
    let codegen = CodegenOptions {
        source_map: true,
        filename: filename.into(),
        ..Default::default()
    };
    let result = compile_sfc_for_adapter(
        &descriptor,
        compile_options,
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        codegen,
        if is_production && !config.ssr {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
    )
    .map_err(|error| error.message)?;

    if !result.errors.is_empty() {
        return Err(result
            .errors
            .iter()
            .map(|error| error.message.as_str())
            .collect::<Vec<_>>()
            .join("\n")
            .into());
    }
    if !result.macro_artifacts.is_empty() {
        return Err(cstr!(
            "{filename}: emitted macro artifacts need a dedicated Parcel adapter"
        ));
    }
    if result.css.is_some() && config.styles == StyleMode::Error {
        return Err(cstr!(
            "{filename}: SFC styles are disabled by the native transformer configuration"
        ));
    }

    let (code, _, map) = finalize_module_output_with_map(result.code, false, result.map);
    let mut code = String::from(code.as_str());
    let mut map = map;
    if has_scoped {
        code = attach_scope(code, scope_id.as_str());
    }

    let assets = if config.transform_asset_urls {
        collect_template_asset_urls(source, None, Some(filename))
    } else {
        Vec::new()
    };
    if !assets.is_empty() {
        let rewritten =
            vize_atelier_sfc::bundler::rewrite_template_asset_references(&code, &assets);
        if rewritten != code {
            code = String::from(rewritten.as_str());
            // Asset substitutions edit generated columns. Until the native ABI
            // exposes mapped edits, dropping this map is safer than chaining a
            // map that points at the wrong authored token.
            map = None;
        }
        let mut imports = String::default();
        for asset in &assets {
            imports.push_str("import ");
            imports.push_str(asset.var_name.as_str());
            imports.push_str(" from ");
            let specifier = cstr!("url:{}", asset.url);
            let quoted =
                serde_json::to_string(specifier.as_str()).map_err(|error| cstr!("{error}"))?;
            imports.push_str(quoted.as_str());
            imports.push_str(";\n");
        }
        let import_lines = imports.lines().count();
        code = imports + code.as_str();
        map = map.map(|value| shift_map(value, import_lines));
    }

    if let Some(css) = result.css.filter(|_| config.styles == StyleMode::Inject) {
        let prefix = style_injection(&css, scope_id.as_str())?;
        let lines = prefix.lines().count();
        code = prefix + code.as_str();
        map = map.map(|value| shift_map(value, lines));
    }

    Ok(CompileOutput {
        code,
        map: map.and_then(|value| {
            serde_json::to_string(&value)
                .ok()
                .map(|value| String::from(value.as_str()))
        }),
        warnings: result
            .warnings
            .into_iter()
            .map(|warning| warning.message)
            .collect(),
    })
}

fn validate_descriptor(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    filename: &str,
) -> Result<(), String> {
    let external_script = descriptor
        .script
        .as_ref()
        .and_then(|block| block.src.as_deref());
    let external_setup = descriptor
        .script_setup
        .as_ref()
        .and_then(|block| block.src.as_deref());
    let external_template = descriptor
        .template
        .as_ref()
        .and_then(|block| block.src.as_deref());
    if external_script.is_some() || external_setup.is_some() || external_template.is_some() {
        return Err(cstr!(
            "{filename}: external script and template blocks are not supported by the native Parcel transformer"
        ));
    }
    if !descriptor.custom_blocks.is_empty() {
        return Err(cstr!(
            "{filename}: custom SFC blocks need a dedicated Parcel transformer"
        ));
    }
    for style in &descriptor.styles {
        if style.src.is_some() {
            return Err(cstr!(
                "{filename}: <style src> needs a Parcel CSS pipeline and is not supported by the native transformer"
            ));
        }
        if style.module.is_some() {
            return Err(cstr!(
                "{filename}: CSS modules need a Parcel CSS pipeline and are not supported by the native transformer"
            ));
        }
        if style.lang.as_deref().is_some_and(|lang| lang != "css") {
            return Err(cstr!(
                "{filename}: style preprocessors need a Parcel CSS pipeline and are not supported by the native transformer"
            ));
        }
        if style.content.contains("@import") {
            return Err(cstr!(
                "{filename}: CSS @import needs a Parcel CSS pipeline and is not supported by the native transformer"
            ));
        }
    }
    Ok(())
}

fn attach_scope(code: String, scope_id: &str) -> String {
    if code.contains("export default _sfc_main") {
        let replacement =
            cstr!("_sfc_main.__scopeId = \"data-v-{scope_id}\";\nexport default _sfc_main");
        let rewritten = code.replacen("export default _sfc_main", replacement.as_str(), 1);
        return String::from(rewritten.as_str());
    }
    if code.contains("export default ") {
        let rewritten = code.replacen("export default ", "const _sfc_main = ", 1);
        return cstr!(
            "{rewritten}\n_sfc_main.__scopeId = \"data-v-{scope_id}\";\nexport default _sfc_main;\n"
        );
    }
    code
}

fn style_injection(css: &str, scope_id: &str) -> Result<String, String> {
    let css = serde_json::to_string(css).map_err(|error| cstr!("{error}"))?;
    let style_id = cstr!("vize-style-{scope_id}");
    let id = serde_json::to_string(style_id.as_str()).map_err(|error| cstr!("{error}"))?;
    Ok(cstr!(
        "export const __vize_css__ = {css};\n\
         const __vize_css_id__ = {id};\n\
         if (typeof document !== \"undefined\") {{\n\
           let style = document.getElementById(__vize_css_id__);\n\
           if (!style) {{\n\
             style = document.createElement(\"style\");\n\
             style.id = __vize_css_id__;\n\
             document.head.appendChild(style);\n\
           }}\n\
           style.textContent = __vize_css__;\n\
         }}\n"
    ))
}

fn shift_map(mut map: serde_json::Value, lines: usize) -> serde_json::Value {
    if lines == 0 {
        return map;
    }
    if let Some(mappings) = map.get_mut("mappings")
        && let Some(value) = mappings.as_str()
    {
        *mappings = serde_json::Value::String([";".repeat(lines).as_str(), value].concat());
    }
    map
}

#[cfg(test)]
mod tests;
