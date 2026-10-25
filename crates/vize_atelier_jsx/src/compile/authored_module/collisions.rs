//! Reject authored lexical bindings that shadow the backend's helper aliases.
//! This uses binding nodes, so property names, comments and strings are harmless.

use super::{JsxComponent, Renderer, error};
use crate::{JsxDiagnostic, JsxLang};
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, Declaration, Function, FunctionType,
    IdentifierReference, Program, Statement,
};
use oxc_ast_visit::Visit;
use oxc_semantic::{Scoping, SemanticBuilder};
use vize_l0::FxHashMap;

pub(super) fn check(
    components: &[JsxComponent],
    spans: &[(u32, u32)],
    preamble: &str,
    authored: &Program<'_>,
    lang: JsxLang,
) -> Result<Vec<Renderer>, JsxDiagnostic> {
    let allocator = oxc_allocator::Allocator::default();
    let generated = crate::parse_module(&allocator, preamble, JsxLang::Jsx);
    super::contexts::check(authored, components)?;
    let mut helpers = Bindings {
        top_level_only: true,
        ..Default::default()
    };
    helpers.visit_program(&generated.program);
    let parsed_components: Vec<_> = components
        .iter()
        .map(|component| crate::parse_module(&allocator, component.code(), lang))
        .collect();
    for ((component, parsed), &(start, end)) in components.iter().zip(&parsed_components).zip(spans)
    {
        if !matches!(component, JsxComponent::Vdom(_)) {
            let mut residual = ResidualJsx(false);
            residual.visit_program(&parsed.program);
            if residual.0 {
                return Err(error(
                    start,
                    end,
                    "Vapor/SSR backend retained JSX inside an attribute expression; this context requires shared IR lowering before module emission",
                ));
            }
        }
        for statement in &parsed.program.body {
            if let Statement::ImportDeclaration(import) = statement {
                helpers.visit_import_declaration(import);
            }
        }
    }
    let mut bindings = Bindings::default();
    bindings.visit_program(authored);
    for (name, &(start, end)) in &bindings.names {
        if helpers.names.contains_key(name) {
            return Err(JsxDiagnostic::error(
                vize_l0::cstr!(
                    "JSX authored binding `{name}` shadows a generated runtime helper; rename the binding"
                ),
                start,
                end,
            ));
        }
    }
    let mut scoping = None;
    parsed_components.iter().zip(spans).map(|(parsed, &(start, end))| {
        let mut generated = Bindings { renderer_only: true, ..Default::default() };
        generated.visit_program(&parsed.program);
        // The emitted renderer is anonymous, so this declaration cannot capture
        // an authored `render` binding. Its parameters and body locals can.
        generated.names.remove("render");
        generated.names.remove("ssrRender");
        let mut missing = find_capture(authored, start, end, &generated.names, scoping.as_ref());
        if missing.is_some() && scoping.is_none() {
            // Most roots cannot collide. Resolve only candidate references, once
            // per authored module, preserving slot/local bindings inside JSX.
            scoping = Some(SemanticBuilder::new().build(authored).semantic.into_scoping());
            missing = find_capture(authored, start, end, &generated.names, scoping.as_ref());
        }
        if let Some((name, start, end)) = missing {
            return Err(JsxDiagnostic::error(
                vize_l0::cstr!("JSX authored reference `{name}` is shadowed by a generated renderer binding; rename the reference"), start, end,
            ));
        }
        renderer(parsed)
    }).collect()
}

#[derive(Default)]
struct Bindings<'a> {
    top_level_only: bool,
    renderer_only: bool,
    in_renderer: bool,
    names: FxHashMap<&'a str, (u32, u32)>,
}

impl<'a> Visit<'a> for Bindings<'a> {
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let renderer = self.renderer_only
            && !self.in_renderer
            && function
                .id
                .as_ref()
                .is_some_and(|id| matches!(id.name.as_str(), "render" | "ssrRender"));
        if self.top_level_only || (self.renderer_only && !renderer) {
            // Nested callbacks own their parameters and locals. Only normal
            // declarations add a binding to the containing renderer scope.
            if function.r#type == FunctionType::FunctionDeclaration
                && let Some(identifier) = &function.id
            {
                self.visit_binding_identifier(identifier);
            }
        } else {
            self.in_renderer = renderer;
            oxc_ast_visit::walk::walk_function(self, function, flags);
            self.in_renderer = false;
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if !self.renderer_only && !self.top_level_only {
            oxc_ast_visit::walk::walk_arrow_function_expression(self, arrow);
        }
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.names.insert(
            identifier.name.as_str(),
            (identifier.span.start, identifier.span.end),
        );
    }
}

fn renderer(parsed: &crate::ParsedModule<'_>) -> Result<Renderer, JsxDiagnostic> {
    if !parsed.has_errors() {
        let imports = parsed
            .program
            .body
            .iter()
            .filter_map(|statement| {
                if let Statement::ImportDeclaration(import) = statement {
                    Some((import.span.start as usize, import.span.end as usize))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for statement in &parsed.program.body {
            let (function, start) = match statement {
                Statement::ExportNamedDeclaration(export) => match &export.declaration {
                    Some(Declaration::FunctionDeclaration(function)) => {
                        (function.as_ref(), export.span.start)
                    }
                    _ => continue,
                },
                Statement::FunctionDeclaration(function) => {
                    (function.as_ref(), function.span.start)
                }
                _ => continue,
            };
            if function
                .id
                .as_ref()
                .is_some_and(|name| matches!(name.name.as_str(), "render" | "ssrRender"))
            {
                return Ok(Renderer {
                    imports,
                    prefix_end: start as usize,
                    params_start: function.params.span.start as usize,
                    params_end: function.params.span.end as usize,
                    body_start: function
                        .body
                        .as_ref()
                        .ok_or_else(|| error(0, 0, "missing JSX render body"))?
                        .span
                        .start as usize,
                    function_end: function.span.end as usize,
                });
            }
        }
    }
    Err(error(
        0,
        0,
        "JSX backend did not produce a supported render declaration",
    ))
}

struct References<'a, 'g> {
    start: u32,
    end: u32,
    generated: &'g FxHashMap<&'g str, (u32, u32)>,
    scoping: Option<&'g Scoping>,
    missing: Option<(&'a str, u32, u32)>,
}

fn find_capture<'a>(
    program: &Program<'a>,
    start: u32,
    end: u32,
    generated: &FxHashMap<&str, (u32, u32)>,
    scoping: Option<&Scoping>,
) -> Option<(&'a str, u32, u32)> {
    let mut references = References {
        start,
        end,
        generated,
        scoping,
        missing: None,
    };
    references.visit_program(program);
    references.missing
}

impl<'a> Visit<'a> for References<'a, '_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.start <= identifier.span.start
            && identifier.span.end <= self.end
            && self.generated.contains_key(identifier.name.as_str())
            && !self.scoping.is_some_and(|scoping| {
                identifier
                    .reference_id
                    .get()
                    .and_then(|id| scoping.get_reference(id).symbol_id())
                    .map(|id| scoping.symbol_span(id))
                    .is_some_and(|binding| self.start <= binding.start && binding.end <= self.end)
            })
        {
            self.missing = Some((
                identifier.name.as_str(),
                identifier.span.start,
                identifier.span.end,
            ));
        }
    }
}

/// A TSX parser can accept residual JSX that no emitted runtime can execute.
/// Inspect syntax nodes, never source spelling, before admitting native output.
struct ResidualJsx(bool);
impl<'a> Visit<'a> for ResidualJsx {
    fn visit_jsx_element(&mut self, _: &oxc_ast::ast::JSXElement<'a>) {
        self.0 = true;
    }
    fn visit_jsx_fragment(&mut self, _: &oxc_ast::ast::JSXFragment<'a>) {
        self.0 = true;
    }
}
