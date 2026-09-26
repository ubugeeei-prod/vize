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
    source: &str,
    lang: JsxLang,
) -> Result<Vec<Renderer>, JsxDiagnostic> {
    let allocator = oxc_allocator::Allocator::default();
    let generated = crate::parse_module(&allocator, preamble, JsxLang::Jsx);
    let authored = crate::parse_module(&allocator, source, lang);
    super::contexts::check(&authored.program, components)?;
    let mut helpers = Bindings {
        top_level_only: true,
        ..Default::default()
    };
    helpers.visit_program(&generated.program);
    let mut bindings = Bindings::default();
    bindings.visit_program(&authored.program);
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
    components.iter().zip(spans).map(|(component, &(start, end))| {
        let allocator = oxc_allocator::Allocator::default();
        let parsed = crate::parse_module(&allocator, component.code(), JsxLang::Jsx);
        let mut generated = Bindings { renderer_only: true, ..Default::default() };
        generated.visit_program(&parsed.program);
        // The emitted renderer is anonymous, so this declaration cannot capture
        // an authored `render` binding. Its parameters and body locals can.
        generated.names.remove("render");
        let mut missing = find_capture(&authored.program, start, end, &generated.names, scoping.as_ref());
        if missing.is_some() && scoping.is_none() {
            // Most roots cannot collide. Resolve only candidate references, once
            // per authored module, preserving slot/local bindings inside JSX.
            scoping = Some(SemanticBuilder::new().build(&authored.program).semantic.into_scoping());
            missing = find_capture(&authored.program, start, end, &generated.names, scoping.as_ref());
        }
        if let Some((name, start, end)) = missing {
            return Err(JsxDiagnostic::error(
                vize_l0::cstr!("JSX authored reference `{name}` is shadowed by a generated renderer binding; rename the reference"), start, end,
            ));
        }
        renderer(&parsed)
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
            && function.id.as_ref().is_some_and(|id| id.name == "render");
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
        for statement in &parsed.program.body {
            if let Statement::ExportNamedDeclaration(export) = statement
                && let Some(Declaration::FunctionDeclaration(function)) = &export.declaration
                && let Some(name) = &function.id
                && name.name == "render"
            {
                return Ok(Renderer {
                    prefix_end: export.span.start as usize,
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
