//! Production JSX backends consuming only owned Rendu/component artifacts.

mod source_map;

use vize_atelier_dom::{DomOutputArtifact, RenduDomMapping, RenduDomOutput};
use vize_atelier_ssr::SsrOutputArtifact;
use vize_atelier_vapor::{VaporOutput, VaporOutputArtifact};
use vize_carton::{String, ToCompactString};

use super::super::JsxRenderModule;
use crate::compile::JsxCompileOutput;
use crate::scoped::build_scoped_style;
use crate::{
    JsxCompileConfig, JsxComponent, JsxOutputMode, SsrComponent, VaporComponent, VdomComponent,
};

pub(super) fn compile_render_module(
    module: &JsxRenderModule,
    dom: Option<&DomOutputArtifact>,
    ssr: Option<&SsrOutputArtifact>,
    vapor: Option<&VaporOutputArtifact>,
    filename: &str,
    source: &str,
    config: &JsxCompileConfig,
) -> Result<JsxCompileOutput, &'static str> {
    let components = module
        .roots
        .iter()
        .enumerate()
        .map(|(index, root)| {
            let mode = root.metadata.mode.unwrap_or(config.default_mode);
            if config.ssr {
                let output = ssr
                    .and_then(|artifact| artifact.outputs().get(index))
                    .ok_or("SSR backend output is missing a JSX component root")?;
                Ok(JsxComponent::Ssr(compile_ssr(root, mode, output)))
            } else {
                match mode {
                    JsxOutputMode::Vdom => dom
                        .and_then(|artifact| artifact.outputs().get(index))
                        .map(|output| {
                            JsxComponent::Vdom(compile_vdom(root, output, filename, source, config))
                        })
                        .ok_or("DOM backend output is missing a JSX component root"),
                    JsxOutputMode::Vapor => vapor
                        .and_then(|artifact| artifact.outputs().get(index))
                        .map(|output| JsxComponent::Vapor(compile_vapor(root, mode, output)))
                        .ok_or("Vapor backend output is missing a JSX component root"),
                }
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(JsxCompileOutput::from_owned_components(
        components,
        source,
        module.diagnostics.clone(),
    ))
}

fn compile_vdom(
    root: &super::super::JsxRenderRoot,
    backend: &RenduDomOutput,
    filename: &str,
    source: &str,
    config: &JsxCompileConfig,
) -> VdomComponent {
    let metadata = &root.metadata;
    let scoped_style = metadata
        .scoped_css
        .as_deref()
        .map(|css| build_scoped_style(metadata.component_name.as_deref(), css));
    let mut output = backend.clone();
    use_element_block_calls(&mut output.code, &mut output.mappings);
    if let Some(style) = &scoped_style {
        inject_scope_id(&mut output.code, &mut output.mappings, &style.scope_id);
    }
    let (preamble, code, body_start) = split_module(output.code);
    let map = config.vdom.source_map.then(|| {
        source_map::from_dom_mappings(
            code.as_str(),
            body_start,
            &output.mappings,
            filename,
            source,
        )
    });
    VdomComponent {
        component_name: metadata.component_name.as_deref().map(String::from),
        component_setup: metadata.component_setup.clone(),
        mode: metadata.mode.unwrap_or(JsxOutputMode::Vdom),
        code,
        preamble,
        map,
        scoped_style,
    }
}

fn compile_ssr(
    root: &super::super::JsxRenderRoot,
    mode: JsxOutputMode,
    backend: &vize_atelier_ssr::RenduSsrOutput,
) -> SsrComponent {
    let metadata = &root.metadata;
    let scoped_style = metadata
        .scoped_css
        .as_deref()
        .map(|css| build_scoped_style(metadata.component_name.as_deref(), css));
    SsrComponent {
        component_name: metadata.component_name.as_deref().map(String::from),
        component_setup: metadata.component_setup.clone(),
        mode,
        code: backend.code.clone(),
        scoped_style,
    }
}

fn compile_vapor(
    root: &super::super::JsxRenderRoot,
    mode: JsxOutputMode,
    backend: &VaporOutput,
) -> VaporComponent {
    let metadata = &root.metadata;
    let scoped_style = metadata
        .scoped_css
        .as_deref()
        .map(|css| build_scoped_style(metadata.component_name.as_deref(), css));
    let mut output = backend.clone();
    if let Some(style) = &scoped_style {
        scope_vapor_templates(&mut output.code, &mut output.templates, &style.scope_id);
    }
    VaporComponent {
        component_name: metadata.component_name.as_deref().map(String::from),
        component_setup: metadata.component_setup.clone(),
        mode,
        code: output.code,
        templates: output.templates,
        scoped_style,
    }
}

fn scope_vapor_templates(code: &mut String, templates: &mut [String], scope_id: &str) {
    for template in templates {
        if !template.contains('<') {
            continue;
        }
        let scoped = add_scope_to_opening_tags(template, scope_id);
        let before = quote_js(template);
        let after = quote_js(&scoped);
        if let Some(start) = code.find(before.as_str()) {
            code.replace_range(start..start + before.len(), after.as_str());
        }
        *template = scoped;
    }
}

fn add_scope_to_opening_tags(html: &str, scope_id: &str) -> String {
    let mut output = String::with_capacity(html.len() + scope_id.len());
    let bytes = html.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let Some(relative) = html[cursor..].find('<') else {
            output.push_str(&html[cursor..]);
            break;
        };
        let start = cursor + relative;
        output.push_str(&html[cursor..start + 1]);
        cursor = start + 1;
        if cursor >= bytes.len() || matches!(bytes[cursor], b'/' | b'!' | b'?') {
            continue;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && !matches!(bytes[cursor], b' ' | b'\t' | b'\n' | b'\r' | b'/' | b'>')
        {
            cursor += 1;
        }
        output.push_str(&html[name_start..cursor]);
        output.push(' ');
        output.push_str(scope_id);
    }
    output
}

fn quote_js(value: &str) -> String {
    let json = serde_json::to_string(value).expect("JSON strings are valid JavaScript strings");
    String::from(json)
}

fn split_module(module: String) -> (String, String, usize) {
    let Some(separator) = module.find("\n\n") else {
        return (String::default(), module, 0);
    };
    let body_start = separator + 2;
    (
        String::from(&module[..separator + 1]),
        String::from(&module[body_start..]),
        body_start,
    )
}

fn use_element_block_calls(code: &mut String, mappings: &mut [RenduDomMapping]) {
    if let Some(import) = code.find("h as _h") {
        insert_mapped(
            code,
            mappings,
            import,
            "createElementBlock as _createElementBlock, ",
        );
    }
    let mut cursor = 0;
    while let Some(relative) = code[cursor..].find("_h(\"") {
        let start = cursor + relative;
        replace_mapped(code, mappings, start, start + 2, "_createElementBlock");
        cursor = start + "_createElementBlock".len();
    }
}

fn inject_scope_id(code: &mut String, mappings: &mut [RenduDomMapping], scope_id: &str) {
    let mut cursor = 0;
    let insertion = format_args!("\"{scope_id}\": \"\", ").to_compact_string();
    while let Some(relative) = code[cursor..].find("_createElementBlock(\"") {
        let call = cursor + relative;
        let Some(object) = code[call..].find(", {") else {
            break;
        };
        let at = call + object + 3;
        insert_mapped(code, mappings, at, insertion.as_str());
        cursor = at + insertion.len();
    }
}

fn insert_mapped(code: &mut String, mappings: &mut [RenduDomMapping], at: usize, value: &str) {
    code.insert_str(at, value);
    shift_mappings(mappings, at, 0, value.len());
}

fn replace_mapped(
    code: &mut String,
    mappings: &mut [RenduDomMapping],
    start: usize,
    end: usize,
    value: &str,
) {
    code.replace_range(start..end, value);
    shift_mappings(mappings, start, end - start, value.len());
}

fn shift_mappings(mappings: &mut [RenduDomMapping], start: usize, removed: usize, inserted: usize) {
    let end = start + removed;
    let delta = inserted as isize - removed as isize;
    for mapping in mappings {
        if mapping.generated_start >= end {
            mapping.generated_start = mapping.generated_start.saturating_add_signed(delta);
        }
        if mapping.generated_end >= end {
            mapping.generated_end = mapping.generated_end.saturating_add_signed(delta);
        }
    }
}
