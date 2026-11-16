//! TSRX modules on the virtual-DOM target (Vue profile).
//!
//! Host code is copied byte for byte. Each component definition becomes
//! `defineComponent(function Name(params) { setup; return (_ctx, _cache) => render }, { props })`:
//! setup statements run once per instance, as in the reference's
//! `defineVaporComponent`, and the render closure is the region emitted by the
//! shared L2 DOM emitter. Markup names are lexical: every name a region reads
//! is a closure binding, except destructured props, which read `__props.key`
//! like the reference's props restructure.

use alloc::vec::Vec;
use vize_l0::pass::NoObserver;
use vize_l0::{Allocator, Span, String};
use vize_l1::markup::tsrx::{Node, TsrxModule};
use vize_l1_to_l2::pass::{TransformProfile, run_transform_with_profile};
use vize_l1_to_l2::{LegacyCaps, Lowered};
use vize_l2::lang::tsrx::TsrxArtifacts;

use super::component::{self, Component, Props};
use super::names::{Names, Use};
use super::{EmitIssue, Failure};
use crate::targets::dom::artifact::{
    BindingKind, BindingTable, DomEmitMode, DomEmitOptions, RuntimeHelpers, emit_dom_with_options,
};

/// Names the module writes besides the runtime helpers.
const GENERATED: [&str; 4] = ["__props", "_ctx", "_cache", "_defineComponent"];

pub(super) fn module<'a>(
    allocator: &'a Allocator,
    module: &'a TsrxModule<'a>,
    artifacts: TsrxArtifacts<'a>,
) -> Result<String, Failure> {
    let source = module.program().source_text;
    let fail = |issue: EmitIssue, span: Span| Failure::emit(issue, span);
    if !module.program().directives.is_empty() || module.program().hashbang.is_some() {
        return Err(fail(EmitIssue::Directive, Span::new(0, 0)));
    }
    let components = component::collect(module).map_err(|(issue, span)| fail(issue, span))?;
    let mut helpers = RuntimeHelpers::default();
    let mut renders = Vec::with_capacity(components.len());
    for (component, region) in components.iter().zip(artifacts.into_regions()) {
        if component.region.span != region.span() {
            return Err(fail(EmitIssue::OutsideComponent, region.span()));
        }
        let names = Names::of_component(source, component.region);
        let Node::CodeBlock(block) = &component.region.node else {
            return Err(fail(EmitIssue::NoRender, component.region.span));
        };
        if block.render.is_none() {
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
    out.push_str("import { defineComponent as _defineComponent");
    for (name, alias) in helpers.imports() {
        out.push_str(", ");
        out.push_str(name);
        out.push_str(" as ");
        out.push_str(alias);
    }
    out.push_str(" } from \"vue\";\n");
    let mut cursor = 0;
    for (component, (names, render)) in components.iter().zip(&renders) {
        out.push_str(slice(source, cursor, component.span.start));
        definition(&mut out, source, component, names, render);
        cursor = component.span.end;
    }
    out.push_str(slice(source, cursor, source.len() as u32));
    Ok(out)
}

/// Lexical names, with destructured props read through `__props`.
fn bindings(names: &Names<'_>, props: &Props<'_>) -> BindingTable {
    let destructured = match props {
        Props::Destructured { keys, .. } => keys.as_slice(),
        _ => &[],
    };
    let free = names.free();
    BindingTable::new(
        free.iter().map(|name| {
            let kind = if destructured.contains(name) {
                BindingKind::Props
            } else {
                BindingKind::SetupConst
            };
            (*name, kind)
        }),
        [],
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
    if component.declaration {
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
    let keys = component.props.keys();
    if !keys.is_empty() {
        out.push_str(", { props: [");
        for (index, key) in keys.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            vize_l0::js_string::push_js_string_literal(out, key);
        }
        out.push_str("] }");
    }
    out.push(')');
    if component.declaration {
        out.push(';');
    }
}

/// `@{ setup <render> }` → `{ setup return (_ctx, _cache) => …; }`, with
/// destructured props in setup read through `__props`.
fn body(
    out: &mut String,
    source: &str,
    component: &Component<'_>,
    names: &Names<'_>,
    render: &str,
) {
    let region = component.region.span;
    let Node::CodeBlock(block) = &component.region.node else {
        return;
    };
    let Some(render_node) = &block.render else {
        return;
    };
    let rendered = render_node.span();
    // Skip the `@` of `@{`.
    let mut cursor = region.start + 1;
    if let Props::Destructured { keys, .. } = &component.props {
        let mut reads: Vec<_> = names
            .reads
            .iter()
            .filter(|read| read.setup && keys.contains(&read.name))
            .collect();
        reads.sort_by_key(|read| read.span.start);
        for read in reads {
            out.push_str(slice(source, cursor, read.span.start));
            if read.use_ == Use::Shorthand {
                out.push_str(read.name);
                out.push_str(": ");
            }
            out.push_str("__props.");
            out.push_str(read.name);
            cursor = read.span.end;
        }
    }
    out.push_str(slice(source, cursor, rendered.start));
    out.push_str("return ");
    out.push_str(render);
    out.push(';');
    out.push_str(slice(source, rendered.end, region.end));
}

fn slice(source: &str, start: u32, end: u32) -> &str {
    source.get(start as usize..end as usize).unwrap_or_default()
}
