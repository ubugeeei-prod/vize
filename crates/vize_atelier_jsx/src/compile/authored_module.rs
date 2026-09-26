//! Assemble VDOM modules at OXC-authored spans, retaining their lexical scopes.

mod collisions;
mod contexts;
mod mapped;
mod unsupported;

use vize_l0::{String, ToCompactString};

use super::JsxComponent;
use crate::{ComponentSetupSpan, JsxDiagnostic, JsxLang, JsxOutputMode};
use mapped::ModuleWriter;

struct Renderer {
    prefix_end: usize,
    params_start: usize,
    params_end: usize,
    body_start: usize,
    function_end: usize,
}

pub(super) fn emit(
    components: &[JsxComponent],
    spans: &[(u32, u32)],
    preamble: &str,
    source: &str,
    lang: JsxLang,
    source_map: bool,
) -> Result<Option<(String, Option<String>)>, JsxDiagnostic> {
    if components.is_empty() {
        let mut writer = ModuleWriter::new(source, components, source_map);
        writer.authored(0, source.len() as u32)?;
        return Ok(Some(writer.finish(if lang.is_typescript() {
            "module.tsx"
        } else {
            "module.jsx"
        })));
    }
    if components.iter().any(|c| c.mode() != JsxOutputMode::Vdom)
        || components.iter().any(|c| matches!(c, JsxComponent::Ssr(_)))
    {
        unsupported::check(components, spans, source, lang)?;
        return Ok(None);
    }
    if components.len() != spans.len() {
        return Err(error(0, 0, "JSX root and component counts differ"));
    }
    let renderers = collisions::check(components, spans, preamble, source, lang)?;
    let mut writer = ModuleWriter::new(source, components, source_map);
    writer.synthetic(preamble);
    writer.synthetic("\n");
    let mut define = String::from("_defineComponent");
    while source.contains(define.as_str()) {
        define.push('_');
    }
    if components.iter().any(|c| c.component_setup().is_some()) {
        writer.synthetic("import { defineComponent as ");
        writer.synthetic(&define);
        writer.synthetic(" } from \"vue\"\n");
    }
    let cx = Context {
        components,
        spans,
        renderers,
        define,
    };
    let mut replacements = Vec::new();
    for (index, (component, &(root_start, root_end))) in components.iter().zip(spans).enumerate() {
        if let Some(setup) = component.component_setup() {
            replacements.push((setup.declaration_start, setup.declaration_end, index, true));
        } else if !components.iter().any(|c| {
            c.component_setup().is_some_and(|setup| {
                setup.declaration_start <= root_start && root_end <= setup.declaration_end
            })
        }) {
            replacements.push((root_start, root_end, index, false));
        }
    }
    replacements.sort_unstable_by_key(|r| r.0);
    let mut cursor = 0;
    for (start, end, index, wrapper) in replacements {
        if start < cursor || end < start || end as usize > source.len() {
            return Err(error(start, end, "overlapping JSX module replacements"));
        }
        writer.authored(cursor, start)?;
        if wrapper {
            cx.wrapper(index, &mut writer)?;
        } else {
            cx.expression(index, true, &mut writer)?;
        }
        cursor = end;
    }
    writer.authored(cursor, source.len() as u32)?;
    Ok(Some(writer.finish(if lang.is_typescript() {
        "module.tsx"
    } else {
        "module.jsx"
    })))
}

struct Context<'a> {
    components: &'a [JsxComponent],
    spans: &'a [(u32, u32)],
    renderers: Vec<Renderer>,
    define: String,
}

impl Context<'_> {
    fn expression(
        &self,
        index: usize,
        invoke: bool,
        writer: &mut ModuleWriter<'_>,
    ) -> Result<(), JsxDiagnostic> {
        let renderer = self
            .renderers
            .get(index)
            .ok_or_else(|| error(0, 0, "missing JSX renderer"))?;
        writer.synthetic("(() => {\n");
        writer.generated(index, 0, renderer.prefix_end)?;
        writer.synthetic("return (");
        writer.generated(index, renderer.params_start, renderer.params_end)?;
        writer.synthetic(" => ");
        writer.generated(index, renderer.body_start, renderer.function_end)?;
        writer.synthetic(if invoke {
            ")(undefined, [])\n})()"
        } else {
            ")\n})()"
        });
        Ok(())
    }

    fn authored_range(
        &self,
        start: u32,
        end: u32,
        wrapper: usize,
        writer: &mut ModuleWriter<'_>,
    ) -> Result<(), JsxDiagnostic> {
        let mut cursor = start;
        for (index, &(root_start, root_end)) in self.spans.iter().enumerate() {
            if index != wrapper && start <= root_start && root_end <= end {
                writer.authored(cursor, root_start)?;
                self.expression(index, true, writer)?;
                cursor = root_end;
            }
        }
        writer.authored(cursor, end)
    }

    fn wrapper(&self, index: usize, writer: &mut ModuleWriter<'_>) -> Result<(), JsxDiagnostic> {
        let component = self
            .components
            .get(index)
            .ok_or_else(|| error(0, 0, "missing JSX component"))?;
        let setup = component
            .component_setup()
            .ok_or_else(|| error(0, 0, "missing JSX setup"))?;
        let name = component
            .component_name()
            .ok_or_else(|| error(0, 0, "missing JSX component name"))?;
        writer.synthetic("const ");
        writer.synthetic(name);
        writer.synthetic(" = ");
        writer.synthetic(&self.define);
        writer.synthetic("({\nname: ");
        writer.synthetic(&js_string(name));
        writer.synthetic(",\n");
        if !setup.destructured_props.is_empty() {
            writer.synthetic("props: [");
            for (i, prop) in setup.destructured_props.iter().enumerate() {
                if i != 0 {
                    writer.synthetic(", ");
                }
                writer.synthetic(&js_string(prop));
            }
            writer.synthetic("],\n");
        }
        if setup.is_async {
            writer.synthetic("async ");
        }
        writer.synthetic("setup");
        if setup.type_params_start != setup.type_params_end {
            writer.synthetic("<");
            writer.authored(setup.type_params_start, setup.type_params_end)?;
            writer.synthetic(">");
        }
        writer.synthetic("(");
        self.authored_range(setup.params_start, setup.params_end, index, writer)?;
        writer.synthetic(") {\n");
        self.authored_range(setup.setup_start, setup.setup_end, index, writer)?;
        writer.synthetic("return ");
        self.expression(index, false, writer)?;
        writer.synthetic("\n}\n});");
        validate_contained_roots(self.spans, setup, index)
    }
}

fn validate_contained_roots(
    spans: &[(u32, u32)],
    setup: &ComponentSetupSpan,
    wrapper: usize,
) -> Result<(), JsxDiagnostic> {
    for (index, &(start, end)) in spans.iter().enumerate() {
        if index != wrapper
            && setup.declaration_start <= start
            && end <= setup.declaration_end
            && !((setup.params_start <= start && end <= setup.params_end)
                || (setup.setup_start <= start && end <= setup.setup_end))
        {
            return Err(error(
                start,
                end,
                "JSX outside the retained setup ranges cannot be replaced safely",
            ));
        }
    }
    Ok(())
}

fn js_string(text: &str) -> String {
    let mut output = String::from("\"");
    for ch in text.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\u{2028}' => output.push_str("\\u2028"),
            '\u{2029}' => output.push_str("\\u2029"),
            _ => output.push(ch),
        }
    }
    output.push('"');
    output
}

fn error(start: u32, end: u32, message: &str) -> JsxDiagnostic {
    JsxDiagnostic::error(message.to_compact_string(), start, end)
}
