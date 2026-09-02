//! Finding outermost JSX render roots and their component context.
//!
//! A JSX/TSX module can embed JSX anywhere an expression is allowed (arrow
//! bodies, `return` statements, ternaries, …). We treat every *outermost* JSX
//! element or fragment — one not nested inside another JSX node — as a render
//! root and lower it immediately, while the OXC node is still live, so no JSX
//! references escape the parse arena.
//!
//! While walking we maintain a stack of enclosing function scopes so each root
//! can record:
//! - the nearest `"use vue:vapor"` / `"use vue:vdom"` directive prologue, and
//! - the enclosing component function's name (`function App` or
//!   `const App = () => …`).

mod signature;

use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, FormalParameters, Function, FunctionBody, JSXElement,
    JSXFragment, Program, Statement, TSTypeParameterDeclaration, VariableDeclaration,
    VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use vize_s0::String;

use self::signature::{destructured_prop_names, formal_parameters_range, type_parameters_range};

use crate::diagnostics::JsxDiagnostic;
use crate::lower::{Lowerer, ScopedStyleExpr};
use crate::mode::{DirectiveKind, JsxOutputMode, classify_directive};
use crate::{ComponentSetupSpan, LoweredRoot, StyleExprSpan};

/// Lower every outermost JSX root in `program` into a [`LoweredRoot`].
pub(crate) fn lower_program_roots<'a>(
    program: &Program<'_>,
    lowerer: &mut Lowerer<'a, '_, '_>,
    default_mode: JsxOutputMode,
) -> std::vec::Vec<LoweredRoot<'a>> {
    let mut collector = RootLowerer {
        lowerer,
        roots: std::vec::Vec::new(),
        scopes: std::vec::Vec::new(),
        pending_name: None,
        pending_declaration_span: None,
        default_mode,
    };
    collector.visit_program(program);
    collector.roots
}

/// An enclosing function scope.
struct FnScope {
    mode: Option<JsxOutputMode>,
    name: Option<String>,
    setup: Option<ComponentSetupSpan>,
}

struct RootLowerer<'l, 'a, 'm, 's> {
    lowerer: &'l mut Lowerer<'a, 'm, 's>,
    roots: std::vec::Vec<LoweredRoot<'a>>,
    scopes: std::vec::Vec<FnScope>,
    /// Name captured from a `const X = ...` declarator, claimed by the next
    /// function/arrow we enter.
    pending_name: Option<String>,
    pending_declaration_span: Option<Span>,
    default_mode: JsxOutputMode,
}

impl RootLowerer<'_, '_, '_, '_> {
    fn current_mode(&self) -> Option<JsxOutputMode> {
        self.scopes.iter().rev().find_map(|scope| scope.mode)
    }

    fn current_name(&self) -> Option<String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.name.clone())
    }

    /// Drain the current root's `<style scoped>` blocks into the raw CSS (for
    /// the scoping backends) and the public interpolation spans (for the type
    /// checker), mapping each internal [`ScopedStyleExpr`] to a [`StyleExprSpan`].
    fn take_scoped_style(&mut self) -> (Option<String>, std::vec::Vec<StyleExprSpan>) {
        match self.lowerer.take_scoped_styles() {
            None => (None, std::vec::Vec::new()),
            Some((css, exprs)) => {
                let spans = exprs
                    .into_iter()
                    .map(
                        |ScopedStyleExpr {
                             content,
                             start,
                             end,
                         }| StyleExprSpan {
                            content,
                            start,
                            end,
                        },
                    )
                    .collect();
                (Some(css), spans)
            }
        }
    }

    fn push_scope(
        &mut self,
        body: Option<&FunctionBody<'_>>,
        name: Option<String>,
        setup: Option<ComponentSetupSpan>,
    ) {
        let mode = body.and_then(|body| self.resolve_body_mode(body));
        self.scopes.push(FnScope { mode, name, setup });
    }

    fn current_setup_for_span(&self, span: Span) -> Option<ComponentSetupSpan> {
        self.scopes.last().and_then(|scope| {
            let setup = scope.setup.as_ref()?;
            (setup.render_start == span.start && setup.render_end == span.end)
                .then(|| setup.clone())
        })
    }

    fn block_body_setup_span(
        &self,
        type_parameters: Option<&TSTypeParameterDeclaration<'_>>,
        params: &FormalParameters<'_>,
        body: &FunctionBody<'_>,
        declaration_span: Span,
        is_async: bool,
    ) -> Option<ComponentSetupSpan> {
        let return_stmt = body.statements.iter().find_map(|stmt| {
            let Statement::ReturnStatement(return_stmt) = stmt else {
                return None;
            };
            let argument = return_stmt.argument.as_ref()?;
            jsx_expression_span(argument).map(|span| (return_stmt.span, span))
        })?;

        let (params_start, params_end) = formal_parameters_range(params);
        let (type_params_start, type_params_end) = type_parameters_range(type_parameters);
        let destructured_props = destructured_prop_names(params);
        Some(ComponentSetupSpan {
            destructured_props,
            declaration_start: declaration_span.start,
            declaration_end: declaration_span.end,
            params_start,
            params_end,
            type_params_start,
            type_params_end,
            is_async,
            setup_start: body.span.start.saturating_add(1),
            setup_end: return_stmt.0.start,
            render_start: return_stmt.1.start,
            render_end: return_stmt.1.end,
        })
    }

    /// Resolve the JSX output mode declared by a function body's directive
    /// prologue, reporting diagnostics for malformed or conflicting directives.
    ///
    /// - A directive that opens with `"use vue:"` but does not name a known mode
    ///   (e.g. `"use vue:vdomx"`) is almost always a typo, so it is reported as
    ///   an error and otherwise ignored.
    /// - Two directives selecting *different* modes in one body conflict; the
    ///   first wins and the later one is reported as an error.
    /// - Unrelated prologues (`"use strict"`, …) are left untouched.
    fn resolve_body_mode(&mut self, body: &FunctionBody<'_>) -> Option<JsxOutputMode> {
        let mut resolved: Option<JsxOutputMode> = None;
        for directive in &body.directives {
            let raw = directive.directive.as_str();
            match classify_directive(raw) {
                DirectiveKind::Mode(mode) => match resolved {
                    None => resolved = Some(mode),
                    Some(existing) if existing != mode => {
                        // Point at the string literal itself, not the whole
                        // statement (which includes the trailing `;`).
                        let loc = self.lowerer.mapper().location(directive.expression.span);
                        self.lowerer.report(JsxDiagnostic::error_at(
                            vize_s0::cstr!(
                                "conflicting JSX mode directives: \"{}\" follows \"{}\" in the \
                                 same component; a component can select only one output mode",
                                mode.directive(),
                                existing.directive()
                            ),
                            &loc,
                        ));
                    }
                    Some(_) => {}
                },
                DirectiveKind::MalformedVue => {
                    let loc = self.lowerer.mapper().location(directive.expression.span);
                    self.lowerer.report(JsxDiagnostic::error_at(
                        vize_s0::cstr!(
                            "unknown JSX mode directive \"{raw}\": expected \"{}\" or \"{}\"",
                            JsxOutputMode::Vdom.directive(),
                            JsxOutputMode::Vapor.directive()
                        ),
                        &loc,
                    ));
                }
                DirectiveKind::Unrelated => {}
            }
        }
        resolved
    }
}

fn jsx_expression_span(expression: &Expression<'_>) -> Option<Span> {
    match expression {
        Expression::JSXElement(_) | Expression::JSXFragment(_) => Some(expression.span()),
        Expression::ParenthesizedExpression(parenthesized) => {
            jsx_expression_span(&parenthesized.expression)
        }
        _ => None,
    }
}

impl<'ast> Visit<'ast> for RootLowerer<'_, '_, '_, '_> {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'ast>) {
        // Capture `const App = ...` so an immediately-initialized function or
        // arrow can adopt the binding name.
        if let Some(name) = it.id.get_identifier_name() {
            self.pending_name = Some(String::from(name.as_str()));
        }
        walk::walk_variable_declarator(self, it);
        self.pending_name = None;
    }

    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'ast>) {
        let previous = self.pending_declaration_span;
        self.pending_declaration_span = (it.declarations.len() == 1).then_some(it.span);
        walk::walk_variable_declaration(self, it);
        self.pending_declaration_span = previous;
    }

    fn visit_function(&mut self, it: &Function<'ast>, flags: ScopeFlags) {
        let name = it
            .id
            .as_ref()
            .map(|id| String::from(id.name.as_str()))
            .or_else(|| self.pending_name.take());
        self.push_scope(it.body.as_deref(), name, None);
        walk::walk_function(self, it, flags);
        self.scopes.pop();
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'ast>) {
        let name = self.pending_name.take();
        let setup = if it.expression {
            None
        } else {
            self.pending_declaration_span.and_then(|span| {
                self.block_body_setup_span(
                    it.type_parameters.as_deref(),
                    &it.params,
                    &it.body,
                    span,
                    it.r#async,
                )
            })
        };
        self.push_scope(Some(&it.body), name, setup);
        walk::walk_arrow_function_expression(self, it);
        self.scopes.pop();
    }

    fn visit_jsx_element(&mut self, element: &JSXElement<'ast>) {
        // Lower this root and intentionally do NOT descend: nested JSX is
        // lowered as part of this root's children, not as separate roots.
        let mode = self.current_mode();
        self.lowerer
            .set_current_output_mode(mode.unwrap_or(self.default_mode));
        let root = self.lowerer.lower_element_root(element);
        let (scoped_css, scoped_style_exprs) = self.take_scoped_style();
        self.roots.push(LoweredRoot {
            root,
            mode,
            component_name: self.current_name(),
            component_setup: self.current_setup_for_span(element.span),
            scoped_css,
            scoped_style_exprs,
        });
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'ast>) {
        let mode = self.current_mode();
        self.lowerer
            .set_current_output_mode(mode.unwrap_or(self.default_mode));
        let root = self.lowerer.lower_fragment_root(fragment);
        let (scoped_css, scoped_style_exprs) = self.take_scoped_style();
        self.roots.push(LoweredRoot {
            root,
            mode,
            component_name: self.current_name(),
            component_setup: self.current_setup_for_span(fragment.span),
            scoped_css,
            scoped_style_exprs,
        });
    }
}
