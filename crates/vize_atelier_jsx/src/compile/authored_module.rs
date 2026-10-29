//! Assemble VDOM modules at OXC-authored spans, retaining their lexical scopes.

mod collisions;
mod component_uses;
mod contexts;
mod legacy_fallback;
mod mapped;
mod native;
mod plans;

use vize_l0::{String, ToCompactString};

use super::JsxComponent;
use crate::{ComponentSetupSpan, JsxDiagnostic, JsxLang};
use mapped::ModuleWriter;

struct Renderer {
    imports: Vec<(usize, usize)>,
    prefix_end: usize,
    params_start: usize,
    params_end: usize,
    body_start: usize,
    function_end: usize,
}

pub(super) fn emit(
    components: &[JsxComponent],
    origins: &[crate::emission::RootSourceMap],
    spans: &[(u32, u32)],
    preamble: &str,
    source: &str,
    lang: JsxLang,
    source_map: bool,
) -> Result<Option<(String, Option<String>)>, JsxDiagnostic> {
    if components.is_empty() {
        let mut writer = ModuleWriter::new(source, components, origins, source_map)?;
        writer.authored(0, source.len() as u32)?;
        return Ok(Some(writer.finish(if lang.is_typescript() {
            "module.tsx"
        } else {
            "module.jsx"
        })));
    }
    if components.len() != spans.len() {
        return Err(error(0, 0, "JSX root and component counts differ"));
    }
    let parse_allocator = oxc_allocator::Allocator::default();
    let parsed = crate::parse_module(&parse_allocator, source, lang);
    let native_roots: Vec<_> = components
        .iter()
        .zip(spans)
        .filter_map(|(component, span)| {
            (!matches!(component, JsxComponent::Vdom(_))).then_some(*span)
        })
        .collect();
    let native_plans = match vize_l0::profile!(
        "atelier.jsx.module.component_plans",
        plans::collect(&parsed.program, &native_roots)
    ) {
        Ok(plans) => plans,
        Err(diagnostic) => {
            if legacy_fallback::admitted(components, spans, source, lang) {
                return Ok(None);
            }
            return Err(diagnostic);
        }
    };
    let profiler = vize_l0::profiler::global_profiler();
    if profiler.is_enabled() {
        profiler
            .record_counter_enabled("atelier.jsx.module.native_roots", native_roots.len() as u64);
        profiler
            .record_counter_enabled("atelier.jsx.module.native_plans", native_plans.len() as u64);
        profiler.record_counter_enabled(
            "atelier.jsx.module.native_generated_maps",
            origins
                .iter()
                .filter(|origin| origin.json.is_some())
                .count() as u64,
        );
    }
    contexts::check_spans(
        &parsed.program,
        &native_plans
            .iter()
            .map(|plan| plan.span)
            .collect::<Vec<_>>(),
    )?;
    if let Some(&(start, end)) = native_roots.iter().find(|&&(start, end)| {
        !native_plans
            .iter()
            .any(|plan| plan.setup.render_start == start && plan.setup.render_end == end)
    }) {
        if legacy_fallback::admitted(components, spans, source, lang) {
            return Ok(None);
        }
        return Err(error(
            start,
            end,
            "Vapor/SSR authored modules require a directly returned JSX root in an owned component arrow; function declarations, callbacks and other expression contexts are not represented yet",
        ));
    }
    for &(start, end) in &native_roots {
        if let Some((index, _)) = spans
            .iter()
            .enumerate()
            .find(|(_, span)| **span == (start, end))
            && matches!(components.get(index), Some(JsxComponent::Ssr(component)) if component.mode == crate::JsxOutputMode::Vdom)
            && native_plans.iter().any(|plan| {
                plan.setup.render_start == start
                    && plan.setup.render_end == end
                    && plan.setup.setup_start == 0
            })
        {
            return Err(error(
                start,
                end,
                "SSR expression-bodied VDOM components require an explicit functional props and attribute fallthrough projection; use a stateful block-body component or the Vapor component role",
            ));
        }
        if native_plans.iter().any(|plan| {
            plan.span.start <= start
                && end <= plan.span.end
                && (plan.setup.render_start, plan.setup.render_end) != (start, end)
        }) {
            return Err(error(
                start,
                end,
                "nested native component roots need an explicit nested setup plan",
            ));
        }
    }
    let renderers = vize_l0::profile!(
        "atelier.jsx.module.collisions",
        collisions::check(
            components,
            spans,
            preamble,
            &parsed.program,
            lang,
            &native_plans
        )
    )?;
    let mut writer = ModuleWriter::new(source, components, origins, source_map)?;
    // Hashbangs and directives retain their original leading position. Imports
    // inserted before them would change parsing or the module's directive role.
    let prefix_end = parsed
        .program
        .directives
        .last()
        .map(|directive| directive.span.end)
        .or_else(|| {
            parsed
                .program
                .hashbang
                .as_ref()
                .map(|hashbang| hashbang.span.end)
        })
        .unwrap_or(0);
    writer.authored(0, prefix_end)?;
    if prefix_end > 0 {
        writer.synthetic("\n");
    }
    let mut backend_imports = Vec::new();
    for (component, renderer) in components.iter().zip(&renderers) {
        for &(start, end) in &renderer.imports {
            backend_imports.push(
                component
                    .code()
                    .get(start..end)
                    .ok_or_else(|| error(0, 0, "invalid generated import span"))?,
            );
        }
    }
    let preamble =
        super::preamble::merge_preambles(std::iter::once(preamble).chain(backend_imports));
    writer.synthetic(&preamble);
    writer.synthetic("\n");
    let mut define = String::from("_defineComponent");
    while source.contains(define.as_str()) {
        define.push('_');
    }
    if components
        .iter()
        .any(|c| matches!(c, JsxComponent::Vdom(_)) && c.component_setup().is_some())
    {
        writer.synthetic("import { defineComponent as ");
        writer.synthetic(&define);
        writer.synthetic(" } from \"vue\"\n");
    }
    let cx = Context {
        components,
        spans,
        renderers,
        define,
        native_plans,
    };
    let mut replacements = Vec::new();
    for (index, (component, &(root_start, root_end))) in components.iter().zip(spans).enumerate() {
        if !matches!(component, JsxComponent::Vdom(_)) {
            let plan = cx
                .native_plan(index)
                .ok_or_else(|| error(root_start, root_end, "missing native component plan"))?;
            replacements.push((plan.span.start, plan.span.end, index, true));
        } else if let Some(setup) = component.component_setup() {
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
    let mut cursor = prefix_end;
    for (start, end, index, wrapper) in replacements {
        if start < cursor || end < start || end as usize > source.len() {
            return Err(error(start, end, "overlapping JSX module replacements"));
        }
        writer.authored(cursor, start)?;
        if wrapper {
            if matches!(components.get(index), Some(JsxComponent::Vdom(_))) {
                cx.wrapper(index, &mut writer)?;
            } else {
                cx.native_wrapper(index, &mut writer)?;
            }
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
    native_plans: Vec<plans::Plan>,
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
        let mut prefix = 0;
        for &(start, end) in &renderer.imports {
            if end > renderer.prefix_end {
                return Err(error(0, 0, "generated import overlaps renderer"));
            }
            writer.generated(index, prefix, start)?;
            prefix = end;
        }
        writer.generated(index, prefix, renderer.prefix_end)?;
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
