use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, Function, FunctionBody, JSXAttributeItem,
    JSXAttributeName, JSXChild, JSXElement, JSXExpression, Program, Statement, VariableDeclaration,
    VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use oxc_syntax::scope::ScopeFlags;
use vize_carton::{String, cstr};

use super::build::SyntaxBuilder;
use super::{JsxSyntaxNode, JsxSyntaxRootMetadata, JsxSyntaxScopedStyle};
use crate::mode::{DirectiveKind, classify_directive};
use crate::{ComponentSetupSpan, JsxDiagnostic, JsxOutputMode, StyleExprSpan};

pub(super) struct CollectedRoots {
    pub(super) roots: Vec<JsxSyntaxNode>,
    pub(super) metadata: Vec<JsxSyntaxRootMetadata>,
    pub(super) diagnostics: Vec<JsxDiagnostic>,
}

struct FnScope {
    mode: Option<JsxOutputMode>,
    name: Option<String>,
    setup: Option<ComponentSetupSpan>,
}

struct RootCollector<'s> {
    builder: SyntaxBuilder<'s>,
    roots: Vec<JsxSyntaxNode>,
    metadata: Vec<JsxSyntaxRootMetadata>,
    diagnostics: Vec<JsxDiagnostic>,
    scopes: Vec<FnScope>,
    pending_name: Option<String>,
    pending_declaration_span: Option<Span>,
}

pub(super) fn collect(source: &str, program: &Program<'_>) -> CollectedRoots {
    let mut collector = RootCollector {
        builder: SyntaxBuilder::new(source),
        roots: Vec::new(),
        metadata: Vec::new(),
        diagnostics: Vec::new(),
        scopes: Vec::new(),
        pending_name: None,
        pending_declaration_span: None,
    };
    collector.visit_program(program);
    CollectedRoots {
        roots: collector.roots,
        metadata: collector.metadata,
        diagnostics: collector.diagnostics,
    }
}

impl RootCollector<'_> {
    fn push_root(&mut self, expression: &Expression<'_>, mut node: JsxSyntaxNode) {
        let span = node.span();
        let styles = collect_styles(expression, self.builder.source);
        remove_scoped_styles(&mut node);
        self.roots.push(node);
        self.metadata.push(JsxSyntaxRootMetadata {
            span,
            mode: self.scopes.iter().rev().find_map(|scope| scope.mode),
            component_name: self
                .scopes
                .iter()
                .rev()
                .find_map(|scope| scope.name.as_ref().map(|name| name.as_str().into())),
            component_setup: self.scopes.last().and_then(|scope| {
                scope.setup.as_ref().and_then(|setup| {
                    (setup.render_start == span.start && setup.render_end == span.end)
                        .then(|| setup.clone())
                })
            }),
            scoped_css: styles.css.map(|css| css.as_str().into()),
            scoped_styles: styles.blocks,
            scoped_style_exprs: styles.expressions,
        });
    }

    fn push_scope(
        &mut self,
        body: Option<&FunctionBody<'_>>,
        name: Option<String>,
        setup: Option<ComponentSetupSpan>,
    ) {
        let mode = body.and_then(|body| self.resolve_mode(body));
        self.scopes.push(FnScope { mode, name, setup });
    }

    fn resolve_mode(&mut self, body: &FunctionBody<'_>) -> Option<JsxOutputMode> {
        let mut resolved = None;
        for directive in &body.directives {
            let raw = directive.directive.as_str();
            match classify_directive(raw) {
                DirectiveKind::Mode(mode) => match resolved {
                    None => resolved = Some(mode),
                    Some(existing) if existing != mode => self.diagnostics.push(
                        JsxDiagnostic::error(
                            cstr!(
                                "conflicting JSX mode directives: \"{}\" follows \"{}\" in the same component; a component can select only one output mode",
                                mode.directive(),
                                existing.directive()
                            ),
                            directive.expression.span.start,
                            directive.expression.span.end,
                        ),
                    ),
                    Some(_) => {}
                },
                DirectiveKind::MalformedVue => self.diagnostics.push(JsxDiagnostic::error(
                    cstr!(
                        "unknown JSX mode directive \"{raw}\": expected \"{}\" or \"{}\"",
                        JsxOutputMode::Vdom.directive(),
                        JsxOutputMode::Vapor.directive()
                    ),
                    directive.expression.span.start,
                    directive.expression.span.end,
                )),
                DirectiveKind::Unrelated => {}
            }
        }
        resolved
    }
}

impl<'ast> Visit<'ast> for RootCollector<'_> {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'ast>) {
        if let Some(name) = declarator.id.get_identifier_name() {
            self.pending_name = Some(String::from(name.as_str()));
        }
        walk::walk_variable_declarator(self, declarator);
        self.pending_name = None;
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'ast>) {
        let previous = self.pending_declaration_span;
        self.pending_declaration_span =
            (declaration.declarations.len() == 1).then_some(declaration.span);
        walk::walk_variable_declaration(self, declaration);
        self.pending_declaration_span = previous;
    }

    fn visit_function(&mut self, function: &Function<'ast>, flags: ScopeFlags) {
        let name = function
            .id
            .as_ref()
            .map(|id| String::from(id.name.as_str()))
            .or_else(|| self.pending_name.take());
        self.push_scope(function.body.as_deref(), name, None);
        walk::walk_function(self, function, flags);
        self.scopes.pop();
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'ast>) {
        let name = self.pending_name.take();
        let setup = (!arrow.expression)
            .then_some(self.pending_declaration_span)
            .flatten()
            .and_then(|declaration| block_setup_span(&arrow.body, declaration));
        self.push_scope(Some(&arrow.body), name, setup);
        walk::walk_arrow_function_expression(self, arrow);
        self.scopes.pop();
    }

    fn visit_expression(&mut self, expression: &Expression<'ast>) {
        if let Some(root) = self.builder.render_expression(expression) {
            self.push_root(expression, root);
        } else {
            walk::walk_expression(self, expression);
        }
    }
}

fn block_setup_span(body: &FunctionBody<'_>, declaration: Span) -> Option<ComponentSetupSpan> {
    let (statement, render) = body.statements.iter().find_map(|statement| {
        let Statement::ReturnStatement(statement) = statement else {
            return None;
        };
        let render = jsx_expression_span(statement.argument.as_ref()?)?;
        Some((statement.span, render))
    })?;
    Some(ComponentSetupSpan {
        declaration_start: declaration.start,
        declaration_end: declaration.end,
        setup_start: body.span.start.saturating_add(1),
        setup_end: statement.start,
        render_start: render.start,
        render_end: render.end,
    })
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

#[derive(Default)]
struct Styles {
    css: Option<String>,
    blocks: Vec<JsxSyntaxScopedStyle>,
    expressions: Vec<StyleExprSpan>,
}

fn collect_styles(expression: &Expression<'_>, source: &str) -> Styles {
    let mut collector = StyleCollector {
        source,
        styles: Styles::default(),
    };
    collector.visit_expression(expression);
    collector.styles
}

struct StyleCollector<'s> {
    source: &'s str,
    styles: Styles,
}

impl StyleCollector<'_> {
    fn append_css(&mut self, value: &str) {
        let css = self.styles.css.get_or_insert_with(String::default);
        if !css.is_empty() {
            css.push('\n');
        }
        css.push_str(value.trim());
    }
}

impl<'ast> Visit<'ast> for StyleCollector<'_> {
    fn visit_jsx_element(&mut self, element: &JSXElement<'ast>) {
        if !is_scoped_style(element) {
            walk::walk_jsx_element(self, element);
            return;
        }
        if let Some(block) = scoped_style_block(element, self.source) {
            self.styles.blocks.push(block);
        }
        let mut css = String::default();
        for child in &element.children {
            match child {
                JSXChild::Text(text) => css.push_str(text.value.as_str()),
                JSXChild::ExpressionContainer(container) => match &container.expression {
                    JSXExpression::StringLiteral(string) => css.push_str(string.value.as_str()),
                    JSXExpression::TemplateLiteral(template) => {
                        for quasi in &template.quasis {
                            css.push_str(
                                quasi
                                    .value
                                    .cooked
                                    .as_ref()
                                    .map_or(quasi.value.raw.as_str(), |value| value.as_str()),
                            );
                        }
                        for expression in &template.expressions {
                            let span = expression.span();
                            self.styles.expressions.push(StyleExprSpan {
                                content: String::from(
                                    &self.source[span.start as usize..span.end as usize],
                                ),
                                start: span.start,
                                end: span.end,
                            });
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        self.append_css(css.as_str());
    }
}

fn scoped_style_block(element: &JSXElement<'_>, source: &str) -> Option<JsxSyntaxScopedStyle> {
    let (start, end) =
        if let [JSXChild::ExpressionContainer(container)] = element.children.as_slice() {
            match &container.expression {
                JSXExpression::TemplateLiteral(template) => (
                    template.quasis.first()?.span.start,
                    template.quasis.last()?.span.end,
                ),
                JSXExpression::StringLiteral(string) => (
                    string.span.start.saturating_add(1),
                    string.span.end.saturating_sub(1),
                ),
                _ => (
                    element.children.first()?.span().start,
                    element.children.last()?.span().end,
                ),
            }
        } else {
            (
                element.children.first()?.span().start,
                element.children.last()?.span().end,
            )
        };
    let start_usize = usize::try_from(start).ok()?.min(source.len());
    let end_usize = usize::try_from(end)
        .ok()?
        .min(source.len())
        .max(start_usize);
    Some(JsxSyntaxScopedStyle {
        css: source.get(start_usize..end_usize)?.into(),
        span: super::JsxSyntaxSpan::new(start, end),
    })
}

fn is_scoped_style(element: &JSXElement<'_>) -> bool {
    let name = match &element.opening_element.name {
        oxc_ast::ast::JSXElementName::Identifier(name) => name.name.as_str(),
        oxc_ast::ast::JSXElementName::IdentifierReference(name) => name.name.as_str(),
        _ => return false,
    };
    name == "style"
        && element.opening_element.attributes.iter().any(|attribute| {
            matches!(attribute, JSXAttributeItem::Attribute(attribute)
                if matches!(&attribute.name, JSXAttributeName::Identifier(name) if name.name.as_str() == "scoped"))
        })
}

fn remove_scoped_styles(node: &mut JsxSyntaxNode) {
    let children = match node {
        JsxSyntaxNode::Element(element) => &mut element.children,
        JsxSyntaxNode::Fragment { children, .. } => children,
        JsxSyntaxNode::If { branches, .. } => {
            for branch in branches {
                remove_from_children(&mut branch.body);
            }
            return;
        }
        JsxSyntaxNode::For { body, .. } => body,
        _ => return,
    };
    remove_from_children(children);
}

fn remove_from_children(children: &mut Vec<JsxSyntaxNode>) {
    children.retain(|child| !matches!(child, JsxSyntaxNode::Element(element)
        if element.name.as_ref() == "style" && element.attributes.iter().any(|attribute|
            matches!(attribute, super::JsxSyntaxAttribute::Attribute { name, .. } if name.as_ref() == "scoped"))));
    for child in children {
        remove_scoped_styles(child);
    }
}
