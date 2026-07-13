//! SFC module assembly over a graph-native Rendu backend result.

use serde_json::{Value, json};
use vize_carton::{String, ToCompactString};

use crate::{SfcCompileOptions, SfcCompileResult, SfcDescriptor, SfcError};

use super::{
    RenderFunctionName, append_component_render_export, compile_styles,
    extract_descriptor_macro_artifacts, extract_normal_script_content, finalize_output_mode,
    generate_scope_id, is_ts_lang, rewrite_default, trim_trailing_newlines,
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
    render: GraphRenderModule,
    mut warnings: Vec<SfcError>,
) -> Result<SfcCompileResult, SfcError> {
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
    finalize_output_mode(&mut code, &mut warnings, &options);
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
        macro_artifacts: extract_descriptor_macro_artifacts(descriptor),
    })
}

fn compile_graph_script(
    descriptor: &SfcDescriptor<'_>,
    options: &SfcCompileOptions,
    component_name: &str,
    vapor: bool,
    filename: &str,
) -> Result<(String, Option<crate::BindingMetadata>), SfcError> {
    let preserve_types = options.script.is_ts || options.template.is_ts;
    if descriptor.script_setup.is_some() {
        let source_filename = options
            .script
            .id
            .as_deref()
            .filter(|filename| !filename.is_empty())
            .unwrap_or(filename);
        let compiled = crate::compile_script::compile_script_from_source(
            descriptor,
            &options.script,
            component_name,
            vapor,
            preserve_types,
            Some(source_filename),
        )?;
        let mut code = String::default();
        if let Some(normal) = descriptor.script.as_ref() {
            let mut content = extract_normal_script_content(
                normal.content.as_ref(),
                is_ts_lang(normal.lang.as_deref()),
                preserve_types,
            );
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
        let (mut code, _) = rewrite_default(
            normal.content.as_ref(),
            "_sfc_main",
            is_ts_lang(normal.lang.as_deref()),
        );
        if is_ts_lang(normal.lang.as_deref()) && !preserve_types {
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
