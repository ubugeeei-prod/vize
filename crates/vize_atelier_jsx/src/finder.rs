//! Lowering every outermost JSX render root with its component context.
//!
//! The shared module walk ([`walk_module`]) finds the roots and their context
//! (the nearest `"use vue:vapor"` / `"use vue:vdom"` prologue and the
//! enclosing component function's name). Each root is lowered as the walk
//! reaches it, while the OXC node is still live, so no JSX references escape
//! the parse arena. Entered functions also record a block-body component's
//! setup span for the module renderer.

pub(crate) mod signature;

use oxc_ast::ast::{ArrowFunctionExpression, Expression, Program, Statement};
use oxc_span::{GetSpan, Span};
use vize_l0::String;

use self::signature::{destructured_prop_names, formal_parameters_range, type_parameters_range};

use crate::diagnostics::JsxDiagnostic;
use crate::lower::{Lowerer, ScopedStyleExpr};
use crate::mode::JsxOutputMode;
use crate::syntax::{PlainRoots, RootProjection};
use crate::{ComponentSetupSpan, LoweredRoot, StyleExprSpan, l2};
use vize_l1_to_l2::framework::vue::jsx::{
    EnteredFunction, FunctionNode, JsxComponent, JsxRoot, ModuleVisitor, PrologueIssue,
    RootContext, walk_module,
};
use vize_l2::component::Exec;
use vize_relief::RootNode;

/// Lower every outermost JSX root in `program` into a [`LoweredRoot`].
pub(crate) fn lower_program_roots<'a, P: RootProjection<'a>>(
    program: &Program<'_>,
    lowerer: &mut Lowerer<'a, '_, '_>,
    default_mode: JsxOutputMode,
) -> std::vec::Vec<P::Root> {
    let mut collector = RootLowerer::<P> {
        lowerer,
        roots: std::vec::Vec::new(),
        setups: std::vec::Vec::new(),
        default_mode,
    };
    walk_module(program, &mut collector);
    collector.roots
}

struct RootLowerer<'l, 'a, 'm, 's, P: RootProjection<'a> = PlainRoots> {
    lowerer: &'l mut Lowerer<'a, 'm, 's>,
    roots: std::vec::Vec<P::Root>,
    /// The setup span of each enclosing function, innermost last.
    setups: std::vec::Vec<Option<ComponentSetupSpan>>,
    default_mode: JsxOutputMode,
}

const fn mode_of(exec: Exec) -> JsxOutputMode {
    match exec {
        Exec::ReRender => JsxOutputMode::Vdom,
        Exec::RunOnce => JsxOutputMode::Vapor,
    }
}

impl<'a, P: RootProjection<'a>> RootLowerer<'_, 'a, '_, '_, P> {
    fn lower_l2(
        &self,
        mode: Option<JsxOutputMode>,
        name: Option<&str>,
        oxc_root: JsxRoot<'_, '_>,
        root: &RootNode<'a>,
    ) -> (Result<l2::JsxL2Root<'a>, l2::L2Refusal>, P::Facts) {
        let exec = match mode.unwrap_or(self.default_mode) {
            JsxOutputMode::Vdom => Exec::ReRender,
            JsxOutputMode::Vapor => Exec::RunOnce,
        };
        let name = name.map(|name| -> &'a str { self.lowerer.bump().alloc_str(name) });
        P::lower(self.lowerer, JsxComponent { name, exec }, oxc_root, root)
    }
}

impl<'a, P: RootProjection<'a>> RootLowerer<'_, 'a, '_, '_, P> {
    fn append_typecheck_roots(&mut self, mode: Option<JsxOutputMode>, name: Option<&str>) {
        for retained in self.lowerer.take_typecheck_roots() {
            let root = retained.root;
            let (scoped_css, scoped_style_exprs) = Self::scoped_style(retained.scoped_style);
            let (l2, syntax) =
                P::project(self.lowerer.bump(), self.lowerer.mapper().source(), &root);
            self.roots.push(P::seal(
                LoweredRoot {
                    root,
                    l2,
                    vdom_value_children: retained.vdom_value_children,
                    mode,
                    component_name: name.map(String::from),
                    component_setup: None,
                    scoped_css,
                    scoped_style_exprs,
                },
                self.lowerer.mapper().source(),
                syntax,
            ));
        }
    }

    /// Drain the current root's `<style scoped>` blocks into the raw CSS (for
    /// the scoping backends) and the public interpolation spans (for the type
    /// checker), mapping each internal [`ScopedStyleExpr`] to a [`StyleExprSpan`].
    fn scoped_style(
        styles: Option<(String, std::vec::Vec<ScopedStyleExpr>)>,
    ) -> (Option<String>, std::vec::Vec<StyleExprSpan>) {
        match styles {
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

    fn current_setup_for_span(&self, span: Span) -> Option<ComponentSetupSpan> {
        self.setups.last().and_then(|setup| {
            let setup = setup.as_ref()?;
            (setup.render_start == span.start && setup.render_end == span.end)
                .then(|| setup.clone())
        })
    }

    /// The setup span of a block-body arrow that initializes `declaration_span`
    /// alone and returns markup.
    fn block_body_setup_span(
        arrow: &ArrowFunctionExpression<'_>,
        declaration_span: Span,
    ) -> Option<ComponentSetupSpan> {
        let (params, body) = (&arrow.params, &arrow.body);
        let return_stmt = body.statements.iter().find_map(|stmt| {
            let Statement::ReturnStatement(return_stmt) = stmt else {
                return None;
            };
            let argument = return_stmt.argument.as_ref()?;
            jsx_expression_span(argument).map(|span| (return_stmt.span, span))
        })?;

        let (params_start, params_end) = formal_parameters_range(params);
        let (type_params_start, type_params_end) =
            type_parameters_range(arrow.type_parameters.as_deref());
        let destructured_props = destructured_prop_names(params);
        Some(ComponentSetupSpan {
            destructured_props,
            declaration_start: declaration_span.start,
            declaration_end: declaration_span.end,
            params_start,
            params_end,
            type_params_start,
            type_params_end,
            is_async: arrow.r#async,
            setup_start: body.span.start.saturating_add(1),
            setup_end: return_stmt.0.start,
            render_start: return_stmt.1.start,
            render_end: return_stmt.1.end,
        })
    }

    /// Report a malformed or conflicting mode directive at the string literal
    /// itself, not the whole statement (which includes the trailing `;`).
    ///
    /// - A directive that opens with `"use vue:"` but does not name a known mode
    ///   (e.g. `"use vue:vdomx"`) is almost always a typo, so it is reported as
    ///   an error and otherwise ignored.
    /// - Two directives selecting *different* modes in one body conflict; the
    ///   first wins and the later one is reported as an error.
    fn report_prologue(&mut self, issue: PrologueIssue<'_>) {
        let (span, message) = match issue {
            PrologueIssue::Conflict {
                span,
                first,
                second,
            } => (
                span,
                vize_l0::cstr!(
                    "conflicting JSX mode directives: \"{}\" follows \"{}\" in the \
                     same component; a component can select only one output mode",
                    mode_of(second).directive(),
                    mode_of(first).directive()
                ),
            ),
            PrologueIssue::Unknown { span, directive } => (
                span,
                vize_l0::cstr!(
                    "unknown JSX mode directive \"{directive}\": expected \"{}\" or \"{}\"",
                    JsxOutputMode::Vdom.directive(),
                    JsxOutputMode::Vapor.directive()
                ),
            ),
        };
        let loc = self
            .lowerer
            .mapper()
            .location(Span::new(span.start, span.end));
        self.lowerer.report(JsxDiagnostic::error_at(message, &loc));
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

impl<'a, 'x, P: RootProjection<'a>> ModuleVisitor<'x> for RootLowerer<'_, 'a, '_, '_, P> {
    fn enter_function(&mut self, function: EnteredFunction<'_, 'x>) {
        let setup = match (function.node, function.declaration) {
            (FunctionNode::Arrow(arrow), Some(declaration)) if !arrow.expression => {
                Self::block_body_setup_span(arrow, Span::new(declaration.start, declaration.end))
            }
            _ => None,
        };
        self.setups.push(setup);
    }

    fn exit_function(&mut self) {
        self.setups.pop();
    }

    fn prologue_issue(&mut self, issue: PrologueIssue<'x>) {
        self.report_prologue(issue);
    }

    fn root(&mut self, oxc_root: JsxRoot<'_, 'x>, context: RootContext<'x>) {
        let mode = context.prologue.map(mode_of);
        self.lowerer
            .set_current_output_mode(mode.unwrap_or(self.default_mode));
        let root = match oxc_root {
            JsxRoot::Element(element) => self.lowerer.lower_element_root(element),
            JsxRoot::Fragment(fragment) => self.lowerer.lower_fragment_root(fragment),
        };
        let (l2, syntax) = self.lower_l2(mode, context.name, oxc_root, &root);
        let (scoped_css, scoped_style_exprs) =
            Self::scoped_style(self.lowerer.take_scoped_styles());
        self.roots.push(P::seal(
            LoweredRoot {
                root,
                l2,
                vdom_value_children: self.lowerer.take_vdom_value_children(),
                mode,
                component_name: context.name.map(String::from),
                component_setup: self
                    .current_setup_for_span(Span::new(context.span.start, context.span.end)),
                scoped_css,
                scoped_style_exprs,
            },
            self.lowerer.mapper().source(),
            syntax,
        ));
        self.append_typecheck_roots(mode, context.name);
    }
}
