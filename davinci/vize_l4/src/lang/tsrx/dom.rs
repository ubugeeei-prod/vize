//! TSRX modules on the virtual-DOM target (Vue profile).
//!
//! Host code is copied byte for byte. Each component definition becomes
//! `defineComponent(function Name(params) { setup; return (_ctx, _cache) => render }, { props })`
//! (an arrow's markup body becomes `(params) => (_ctx, _cache) => render`):
//! setup statements run once per instance, as in the reference's
//! `defineVaporComponent`, and the render closure is the region emitted by the
//! shared L2 DOM emitter. Markup names are lexical: every name a region reads
//! is a closure binding, except destructured props, which read `__props.key`
//! like the reference's props restructure.

use alloc::vec::Vec;
use vize_l0::pass::NoObserver;
use vize_l0::{Allocator, Span, String};
use vize_l1::markup::tsrx::TsrxModule;
use vize_l1_to_l2::pass::{TransformProfile, run_transform_with_profile};
use vize_l1_to_l2::{LegacyCaps, Lowered};
use vize_l2::lang::tsrx::TsrxL2Templates;

use super::component::{self, Component, Form};
use super::names::{Names, Use};
use super::props::{Options, Props};
use super::{EmitIssue, Failure};
use crate::targets::dom::artifact::{
    BindingKind, BindingTable, DomEmitMode, DomEmitOptions, RuntimeHelpers, emit_dom_with_options,
};

/// Names the module writes besides the runtime helpers.
const GENERATED: [&str; 5] = ["__props", "_ctx", "_cache", "_defineComponent", "_useAttrs"];

pub(super) fn module<'a>(
    allocator: &'a Allocator,
    module: &'a TsrxModule<'a>,
    artifacts: TsrxL2Templates<'a>,
) -> Result<String, Failure> {
    let source = module.program().source_text;
    let fail = |issue: EmitIssue, span: Span| Failure::emit(issue, span);
    let components = component::collect(module, artifacts.regions())
        .map_err(|(issue, span)| fail(issue, span))?;
    let mut helpers = RuntimeHelpers::default();
    let mut renders = Vec::with_capacity(components.len());
    for (component, region) in components.iter().zip(artifacts.into_regions()) {
        if component.region.span != region.span() {
            return Err(fail(EmitIssue::OutsideComponent, region.span()));
        }
        let names = Names::of_component(module, component);
        if component.render().is_none() {
            return Err(fail(EmitIssue::NoRender, component.region.span));
        }
        let table = bindings(&names, &component.props);
        let mut lowered = Lowered {
            allocator,
            op_count: region.artifact().node_count(),
            artifact: region.into_artifact().into_parts(),
            diagnostics: Vec::new(),
            consumed_expressions: Vec::new(),
            caps: LegacyCaps::VUE3,
        };
        let facts = run_transform_with_profile(
            &mut lowered,
            &mut NoObserver,
            TransformProfile::DEFAULT
                .without_static_analysis()
                .without_complexity_analysis(),
        );
        let emit = emit_dom_with_options(
            &lowered,
            &facts,
            &DomEmitOptions {
                mode: DomEmitMode::Module,
                prefix_identifiers: true,
                hoist_static: false,
                inline: true,
                component_name: Some(component.name),
                bindings: Some(&table),
                // Markup expressions are TypeScript; without the feature the
                // emitter refuses the ones that are not JavaScript.
                is_ts: cfg!(feature = "typescript"),
                ..DomEmitOptions::DEFAULT
            },
        )
        .map_err(|_| fail(EmitIssue::Dom, component.region.span))?;
        // Module-level hoists would be lost; inline emission never makes them.
        if emit.preamble.len() != emit.sections.imports_len {
            return Err(fail(EmitIssue::Dom, component.region.span));
        }
        helpers = helpers.union(emit.helpers);
        let code = emit.code.as_str();
        let sections = emit.sections;
        let mut render = String::from("(_ctx, _cache) => {");
        render.push_str(
            code.get(sections.assets_start..sections.assets_end)
                .unwrap_or_default(),
        );
        render.push_str("\n  return ");
        render.push_str(
            code.get(sections.return_expr_start..sections.return_expr_end)
                .unwrap_or_default(),
        );
        render.push_str("\n}");
        renders.push((names, render));
    }
    let mut used = Names::of_program(module.program());
    used.extend(renders.iter().flat_map(|(names, _)| names.free()));
    if used
        .iter()
        .any(|name| GENERATED.contains(name) || helpers.imports().any(|(_, alias)| alias == *name))
    {
        return Err(fail(EmitIssue::GeneratedName, Span::new(0, 0)));
    }

    let mut out = String::with_capacity(source.len() * 2);
    // A hashbang and the directive prologue stay first, so directives keep
    // their meaning (#13131); the runtime import follows their line.
    let mut cursor = prologue_end(module.program());
    out.push_str(slice(source, 0, cursor));
    if cursor > 0 && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("import { defineComponent as _defineComponent");
    if components
        .iter()
        .any(|component| component.props.rest().is_some())
    {
        out.push_str(", useAttrs as _useAttrs");
    }
    for (name, alias) in helpers.imports() {
        out.push_str(", ");
        out.push_str(name);
        out.push_str(" as ");
        out.push_str(alias);
    }
    out.push_str(" } from \"vue\";\n");
    for (component, (names, render)) in components.iter().zip(&renders) {
        out.push_str(slice(source, cursor, component.span.start));
        definition(&mut out, source, component, names, render);
        cursor = component.span.end;
    }
    out.push_str(slice(source, cursor, source.len() as u32));
    Ok(out)
}

/// Where the line of the hashbang or the last directive ends (after its
/// line break, unless a statement starts on that line), or 0.
fn prologue_end(program: &oxc_ast::ast::Program<'_>) -> u32 {
    let end = program
        .directives
        .last()
        .map(|directive| directive.span.end)
        .or(program.hashbang.as_ref().map(|hashbang| hashbang.span.end))
        .unwrap_or(0);
    if end == 0 {
        return 0;
    }
    let rest = program.source_text.get(end as usize..).unwrap_or_default();
    let line = match rest.find('\n') {
        Some(line) => end + line as u32 + 1,
        None => program.source_text.len() as u32,
    };
    match program.body.first() {
        Some(first) if oxc_span::GetSpan::span(first).start < line => end,
        _ => line,
    }
}

/// Lexical names, with destructured props read through `__props` and a
/// renamed prop read as its key.
fn bindings(names: &Names<'_>, props: &Props<'_>) -> BindingTable {
    let free = names.free();
    let kind = |name: &str| match props.local(name) {
        Some(prop) if prop.key == name => BindingKind::Props,
        Some(_) => BindingKind::PropsAliased,
        None => BindingKind::SetupConst,
    };
    BindingTable::new(
        free.iter().map(|name| (*name, kind(name))),
        free.iter().filter_map(|name| {
            props
                .local(name)
                .filter(|prop| prop.key != prop.local)
                .map(|prop| (prop.local, prop.key))
        }),
        true,
    )
}

fn definition(
    out: &mut String,
    source: &str,
    component: &Component<'_>,
    names: &Names<'_>,
    render: &str,
) {
    if component.form == Form::Declaration {
        out.push_str("const ");
        out.push_str(component.name);
        out.push_str(" = ");
    }
    out.push_str("/* @__PURE__ */ _defineComponent(");
    out.push_str(slice(source, component.head.start, component.head.end));
    let params = component.params.span;
    match &component.props {
        Props::Destructured { param, .. } => {
            out.push_str(slice(source, params.start, param.start));
            out.push_str("__props");
            out.push_str(slice(source, param.end, params.end));
        }
        Props::None | Props::Identifier { .. } => {
            out.push_str(slice(source, params.start, params.end))
        }
    }
    out.push_str(slice(source, component.tail.start, component.tail.end));
    body(out, source, component, names, render);
    options(out, &component.props);
    out.push(')');
    if component.form != Form::Initializer {
        out.push(';');
    }
}

/// `, { inheritAttrs: false, props: … }`: a list of keys, or an object when a
/// prop has runtime options.
fn options(out: &mut String, props: &Props<'_>) {
    let declared = props.declared();
    let inherit = props.rest().is_none();
    if declared.is_empty() && inherit {
        return;
    }
    out.push_str(", { ");
    if !inherit {
        out.push_str("inheritAttrs: false");
        if !declared.is_empty() {
            out.push_str(", ");
        }
    }
    if !declared.is_empty() {
        let object = declared.iter().any(|(_, options)| options.is_some());
        out.push_str(if object { "props: { " } else { "props: [" });
        for (index, (key, options)) in declared.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            vize_l0::js_string::push_js_string_literal(out, key);
            if object {
                out.push_str(": ");
                match options {
                    Some(options) => prop_options(out, options),
                    None => out.push_str("null"),
                }
            }
        }
        out.push_str(if object { " }" } else { "]" });
    }
    out.push_str(" }");
}

fn prop_options(out: &mut String, options: &Options<'_>) {
    out.push_str("{ ");
    if options.required {
        out.push_str("required: true, ");
    }
    if let Some(type_) = options.type_ {
        out.push_str("type: ");
        out.push_str(type_);
        out.push_str(", ");
    }
    out.push_str("default: ");
    if options.factory {
        out.push_str("() => (");
        out.push_str(options.default);
        out.push(')');
    } else {
        out.push_str(options.default);
    }
    if options.skip_factory {
        out.push_str(", skipFactory: true");
    }
    out.push_str(" }");
}

/// `@{ setup <render> }` → `{ setup return (_ctx, _cache) => …; }`, and
/// `{ setup return <render>; }` → `{ setup return (_ctx, _cache) => …; }`,
/// with destructured props in setup read through `__props` and a rest
/// element bound first, to the attributes.
fn body(
    out: &mut String,
    source: &str,
    component: &Component<'_>,
    names: &Names<'_>,
    render: &str,
) {
    let Some(render_node) = component.render() else {
        return;
    };
    let region = component.region.span;
    // The replaced range, its replacement's prefix, and the copied range.
    let (rendered, prefix, start, end) = match component.body {
        // Skip the `@` of `@{`.
        None => (render_node.span(), "return ", region.start + 1, region.end),
        Some(_) => (region, "", component.tail.end, component.span.end),
    };
    let mut cursor = start;
    if let Some(rest) = component.props.rest() {
        // The copied range opens the body's `{`.
        cursor += 1;
        out.push_str("{ const ");
        out.push_str(rest);
        out.push_str(" = _useAttrs();");
    }
    let mut reads: Vec<_> = names
        .reads
        .iter()
        .filter(|read| read.setup && !read.shadowed)
        .filter_map(|read| Some((read, component.props.local(read.name)?)))
        .collect();
    reads.sort_by_key(|(read, _)| read.span.start);
    for (read, prop) in reads {
        out.push_str(slice(source, cursor, read.span.start));
        if read.use_ == Use::Shorthand {
            out.push_str(read.name);
            out.push_str(": ");
        }
        out.push_str("__props.");
        out.push_str(prop.key);
        cursor = read.span.end;
    }
    out.push_str(slice(source, cursor, rendered.start));
    out.push_str(prefix);
    out.push_str(render);
    if component.body.is_none() {
        out.push(';');
    }
    out.push_str(slice(source, rendered.end, end));
}

fn slice(source: &str, start: u32, end: u32) -> &str {
    source.get(start as usize..end as usize).unwrap_or_default()
}
