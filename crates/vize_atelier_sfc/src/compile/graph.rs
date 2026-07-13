//! SFC module assembly over a graph-native Rendu backend result.

use serde_json::{Value, json};
use vize_carton::{String, ToCompactString};

use crate::{SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError};

use super::{
    RenderFunctionName, append_component_render_export, compile_styles,
    extract_descriptor_macro_artifacts, finalize_output_mode, generate_scope_id, is_ts_lang,
    trim_trailing_newlines,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct GraphRenderMapping {
    pub(crate) generated_start: usize,
    pub(crate) source_start: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct GraphRenderModule {
    pub(crate) code: String,
    pub(crate) templates: Option<Vec<String>>,
    pub(crate) mappings: Vec<GraphRenderMapping>,
    pub(crate) render: Option<RenderFunctionName>,
    pub(crate) vapor: bool,
}

pub(crate) fn compile_sfc_with_graph_render(
    descriptor: &SfcDescriptor<'_>,
    options: SfcCompileOptions,
    render_emit: &vize_rendu::RenderEmitSettings,
    render: GraphRenderModule,
    mut warnings: Vec<SfcError>,
    modules: Option<&vize_module::ModuleDocument>,
    script_syntax: Option<&crate::SfcScriptSyntaxSnapshot>,
) -> Result<SfcCompileResult, SfcError> {
    validate_script_modules(descriptor, modules)?;
    let filename = if options.parse.filename.is_empty() {
        options.script.id.as_deref().unwrap_or("anonymous.vue")
    } else {
        options.parse.filename.as_str()
    };
    let component_name = super::extract_component_name(filename);
    let has_scoped = descriptor.styles.iter().any(|style| style.scoped);
    let needs_scope_id = !descriptor.styles.is_empty()
        || !descriptor.css_vars.is_empty()
        || options.scope_id.is_some();
    let scope_id = if needs_scope_id {
        options
            .scope_id
            .clone()
            .unwrap_or_else(|| generate_scope_id(filename))
    } else {
        String::default()
    };
    let styles = compile_styles(&descriptor.styles, &scope_id, &options.style, &mut warnings);
    let css = (!styles.css.is_empty()).then(|| styles.css.clone());
    let (mut code, bindings) = compile_graph_script(
        descriptor,
        &options,
        component_name.as_str(),
        render.vapor,
        filename,
        modules,
        script_syntax,
    )?;
    let render_offset = code.len();
    code.push_str(render.code.as_str());
    if !code.ends_with('\n') {
        code.push('\n');
    }
    if render.vapor {
        code.push_str("_sfc_main.__vapor = true\n");
    }
    if has_scoped {
        code.push_str("_sfc_main.__scopeId = \"data-v-");
        code.push_str(scope_id.as_str());
        code.push_str("\"\n");
    }
    if let Some(render_function) = render.render {
        append_component_render_export(
            &mut code,
            "_sfc_main",
            render_function,
            &styles.css_modules,
        );
    } else {
        super::append_css_modules_assignment(&mut code, "_sfc_main", &styles.css_modules);
        code.push_str("export default _sfc_main\n");
    }
    finalize_output_mode(
        &mut code,
        &mut warnings,
        &options,
        &vize_relief::CodegenOptions {
            runtime_module_name: render_emit.runtime_module_name.clone(),
            runtime_global_name: render_emit.runtime_global_name.clone(),
            ..Default::default()
        },
    );
    trim_trailing_newlines(&mut code);
    let map = source_map(
        &render,
        render_offset,
        code.as_str(),
        descriptor,
        filename,
        source_map_requested(&options),
    );

    Ok(SfcCompileResult {
        code,
        css,
        map,
        errors: Vec::new(),
        warnings,
        bindings,
        macro_artifacts: script_syntax
            .map(crate::SfcScriptSyntaxSnapshot::macro_artifacts)
            .unwrap_or_else(|| extract_descriptor_macro_artifacts(descriptor)),
    })
}

fn compile_graph_script(
    descriptor: &SfcDescriptor<'_>,
    options: &SfcCompileOptions,
    component_name: &str,
    vapor: bool,
    filename: &str,
    modules: Option<&vize_module::ModuleDocument>,
    script_syntax: Option<&crate::SfcScriptSyntaxSnapshot>,
) -> Result<(String, Option<crate::BindingMetadata>), SfcError> {
    let preserve_types = options.script.is_ts || options.template.is_ts;
    if descriptor.script_setup.is_some() {
        let source_filename = options
            .script
            .id
            .as_deref()
            .filter(|filename| !filename.is_empty())
            .unwrap_or(filename);
        let projection = script_syntax
            .and_then(crate::SfcScriptSyntaxSnapshot::setup_compiler)
            .ok_or_else(|| script_module_error("SFC script compiler projection is missing"))?;
        let compiled = crate::compile_script::compile_preanalyzed_script_setup(
            projection,
            component_name,
            vapor,
            preserve_types,
            descriptor
                .template
                .as_ref()
                .map(|template| template.content.as_ref()),
            Some(source_filename),
        )?;
        let mut code = String::default();
        if descriptor.script.is_some() {
            let normal_projection = script_syntax
                .and_then(crate::SfcScriptSyntaxSnapshot::normal_compiler)
                .ok_or_else(|| {
                    script_module_error("normal script compiler projection is missing")
                })?;
            let mut content = String::from(normal_projection.dual_content(preserve_types));
            if !content.contains("const __default__") {
                content.push_str("\nconst __default__ = {}\n");
            }
            code.push_str(content.as_str());
            code.push('\n');
        }
        code.push_str(compiled.code.as_str());
        code.push_str("\nconst _sfc_main = ");
        if descriptor.script.is_some() {
            code.push_str("Object.assign(__default__, __sfc__)\n");
        } else {
            code.push_str("__sfc__\n");
        }
        return Ok((code, compiled.bindings));
    }

    if let Some(normal) = descriptor.script.as_ref() {
        let normal_projection = script_syntax
            .and_then(crate::SfcScriptSyntaxSnapshot::normal_compiler)
            .ok_or_else(|| script_module_error("normal script compiler projection is missing"))?;
        let normal_is_ts =
            module_is_ts(modules, "script").unwrap_or_else(|| is_ts_lang(normal.lang.as_deref()));
        let mut code = String::from(normal_projection.rewritten_default());
        if normal_is_ts && !preserve_types {
            code = crate::compile_script::typescript::transform_typescript_to_js(&code);
        }
        code.push('\n');
        return Ok((code, None));
    }

    let component = if vapor {
        "const _sfc_main = { __vapor: true }\n"
    } else {
        "const _sfc_main = {}\n"
    };
    Ok((component.to_compact_string(), None))
}

fn validate_script_modules(
    descriptor: &SfcDescriptor<'_>,
    modules: Option<&vize_module::ModuleDocument>,
) -> Result<(), SfcError> {
    let expected =
        usize::from(descriptor.script.is_some()) + usize::from(descriptor.script_setup.is_some());
    if expected == 0 {
        return Ok(());
    }
    let Some(modules) = modules else {
        return Err(script_module_error("SFC script module product is missing"));
    };
    if modules.modules.len() != expected {
        return Err(script_module_error(
            "SFC script module count is inconsistent",
        ));
    }
    for (role, block) in [
        ("script", descriptor.script.as_ref()),
        ("script-setup", descriptor.script_setup.as_ref()),
    ] {
        let Some(block) = block else { continue };
        if module_source(Some(modules), role) != Some(block.content.as_ref()) {
            return Err(script_module_error(
                "SFC script module bytes are inconsistent",
            ));
        }
    }
    Ok(())
}

fn module_source<'a>(
    modules: Option<&'a vize_module::ModuleDocument>,
    role: &str,
) -> Option<&'a str> {
    let suffix = vize_carton::cstr!("#{role}");
    modules?
        .modules
        .iter()
        .find(|module| module.name.ends_with(suffix.as_str()))
        .map(|module| module.source.as_ref())
}

fn module_is_ts(modules: Option<&vize_module::ModuleDocument>, role: &str) -> Option<bool> {
    let suffix = vize_carton::cstr!("#{role}");
    modules?
        .modules
        .iter()
        .find(|module| module.name.ends_with(suffix.as_str()))
        .map(|module| {
            matches!(
                module.language,
                vize_module::ModuleLanguage::TypeScript | vize_module::ModuleLanguage::Tsx
            )
        })
}

fn script_module_error(message: &str) -> SfcError {
    SfcError {
        message: message.into(),
        code: Some("INCONSISTENT_SCRIPT_MODULE_ARTIFACTS".into()),
        loc: None,
    }
}

fn source_map_requested(options: &SfcCompileOptions) -> bool {
    options.parse.source_map
        || options.style.source_map
        || options
            .template
            .compiler_options
            .as_ref()
            .is_some_and(|compiler| compiler.source_map)
}

fn source_map(
    render: &GraphRenderModule,
    render_offset: usize,
    final_code: &str,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
    requested: bool,
) -> Option<Value> {
    let template = descriptor.template.as_ref()?;
    if !requested || render.mappings.is_empty() {
        return None;
    }
    let mut segments = Vec::with_capacity(render.mappings.len());
    for mapping in &render.mappings {
        let generated = render_offset.saturating_add(mapping.generated_start);
        let original = template
            .loc
            .start
            .saturating_add(mapping.source_start as usize);
        let (generated_line, generated_column) = line_column(final_code, generated);
        let (original_line, original_column) = line_column(descriptor.source.as_ref(), original);
        segments.push(super::source_maps::vlq::MappingSegment {
            generated_line,
            generated_column,
            original: Some(super::source_maps::vlq::OriginalPosition {
                source: 0,
                line: original_line as i64,
                column: original_column as i64,
                name: None,
            }),
        });
    }
    segments.sort_by_key(|segment| (segment.generated_line, segment.generated_column));
    segments.dedup_by_key(|segment| (segment.generated_line, segment.generated_column));
    Some(json!({
        "version": 3,
        "file": filename,
        "sources": [filename],
        "sourcesContent": [descriptor.source.as_ref()],
        "names": [],
        "mappings": super::source_maps::vlq::encode_mappings(&segments).as_str(),
    }))
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = source[..line_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    (line, source[line_start..offset].encode_utf16().count())
}
