//! TSRX components of the Vue profile, read from the host program.
//!
//! A component is a function whose body is a statement container (`@{ … }`),
//! or that returns markup (`{ setup; return <markup>; }`, `() => <markup>`),
//! and whose static name starts with an uppercase ASCII letter: a function
//! declaration (optionally the named default export), or a `const`/`let`
//! declarator initialized with an arrow or a function expression, optionally
//! exported. An anonymous default export is not a component in the reference
//! (it runs as a functional component whose destructured props never update)
//! and stays refused. Setup returns nothing but the
//! render: an earlier `return` would decide the output once, at setup, and is
//! refused (`EarlyReturn`). Its props follow the pinned reference
//! ([`super::props`]).

use alloc::vec::Vec;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, Class, Declaration, ExportDefaultDeclarationKind,
    Expression, FormalParameters, Function, FunctionBody, ReturnStatement, Statement,
    VariableDeclaration,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;
use vize_l0::Span;
use vize_l1::markup::tsrx::{Node, Region, RegionSite, TsrxModule};
use vize_l1_to_l2::framework::vue::tsrx::TsrxRegion;

use super::EmitIssue;
use super::props::{self, Props};

/// One component whose body region is emitted.
pub(super) struct Component<'a> {
    pub name: &'a str,
    /// The host range the definition replaces.
    pub span: Span,
    /// How the definition replaces its host range.
    pub form: Form,
    /// From the function start through the parameter list's `(`.
    pub head: Span,
    pub params: &'a FormalParameters<'a>,
    /// From the parameter list's `)` through the body: the `@` of a `@{ … }`
    /// region, or the start of a host body.
    pub tail: Span,
    pub region: &'a Region<'a>,
    /// The host body whose return value is the region, `None` for `@{ … }`.
    pub body: Option<&'a FunctionBody<'a>>,
    /// An arrow whose body is its returned expression.
    pub expression: bool,
    pub props: Props<'a>,
}

impl<'a> Component<'a> {
    /// The rendered node: the returned region, or a `@{ … }` body's output.
    pub(super) fn render(&self) -> Option<&'a Node<'a>> {
        match (self.body, &self.region.node) {
            (Some(_), node) => Some(node),
            (None, Node::CodeBlock(block)) => block.render.as_ref(),
            (None, _) => None,
        }
    }
}

/// Where a component is defined.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Form {
    /// A function declaration, which becomes a `const` declaration.
    Declaration,
    /// A declarator's initializer, which stays an expression.
    Initializer,
    /// `export default function Name`, which exports the expression.
    ExportDefault,
}

/// Every top-level component of the module, in source order. A lowered
/// region (`regions`) that is not a component body is refused.
pub(super) fn collect<'a>(
    module: &'a TsrxModule<'a>,
    regions: &[TsrxRegion<'_>],
) -> Result<Vec<Component<'a>>, (EmitIssue, Span)> {
    let mut found = Vec::new();
    for statement in &module.program().body {
        match statement {
            Statement::FunctionDeclaration(function) => {
                declaration(module, function, Form::Declaration, &mut found)?;
            }
            Statement::VariableDeclaration(variables) => {
                declarators(module, variables, &mut found)?;
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(function) =
                    &export.declaration
                {
                    declaration(module, function, Form::ExportDefault, &mut found)?;
                }
            }
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::FunctionDeclaration(function)) => {
                    declaration(module, function, Form::Declaration, &mut found)?;
                }
                Some(Declaration::VariableDeclaration(variables)) => {
                    declarators(module, variables, &mut found)?;
                }
                _ => {}
            },
            _ => {}
        }
    }
    for region in regions {
        if !found
            .iter()
            .any(|component| component.region.span == region.span())
        {
            return Err((EmitIssue::OutsideComponent, region.span()));
        }
    }
    Ok(found)
}

fn declaration<'a>(
    module: &'a TsrxModule<'a>,
    function: &'a Function<'a>,
    form: Form,
    found: &mut Vec<Component<'a>>,
) -> Result<(), (EmitIssue, Span)> {
    let (Some(id), Some(body)) = (&function.id, &function.body) else {
        return Ok(());
    };
    let span = span(function.span);
    if let Some(component) = component(
        module,
        id.name.as_str(),
        span,
        &function.params,
        body,
        false,
        function.r#async || function.generator,
    )? {
        found.push(Component { form, ..component });
    }
    Ok(())
}

fn declarators<'a>(
    module: &'a TsrxModule<'a>,
    variables: &'a VariableDeclaration<'a>,
    found: &mut Vec<Component<'a>>,
) -> Result<(), (EmitIssue, Span)> {
    for declarator in &variables.declarations {
        let BindingPattern::BindingIdentifier(id) = &declarator.id else {
            continue;
        };
        let (start, params, body, expression, unsupported) = match &declarator.init {
            Some(Expression::ArrowFunctionExpression(arrow)) => (
                arrow.span,
                &*arrow.params,
                &*arrow.body,
                arrow.expression,
                arrow.r#async,
            ),
            Some(Expression::FunctionExpression(function)) => match &function.body {
                Some(body) => (
                    function.span,
                    &*function.params,
                    &**body,
                    false,
                    function.r#async || function.generator,
                ),
                None => continue,
            },
            _ => continue,
        };
        if let Some(component) = component(
            module,
            id.name.as_str(),
            span(start),
            params,
            body,
            expression,
            unsupported,
        )? {
            found.push(component);
        }
    }
    Ok(())
}

fn component<'a>(
    module: &'a TsrxModule<'a>,
    name: &'a str,
    span: Span,
    params: &'a FormalParameters<'a>,
    body: &'a FunctionBody<'a>,
    expression: bool,
    unsupported: bool,
) -> Result<Option<Component<'a>>, (EmitIssue, Span)> {
    let (region, host_body) = match module
        .body_region(body)
        .filter(|region| region.site == RegionSite::FunctionBody)
    {
        Some(region) => (region, None),
        None => match returned_region(module, body, expression) {
            Some(region) => (region, Some(body)),
            None => return Ok(None),
        },
    };
    if !name.starts_with(|c: char| c.is_ascii_uppercase()) {
        if host_body.is_some() {
            return Ok(None);
        }
        return Err((EmitIssue::OutsideComponent, region.span));
    }
    if unsupported {
        return Err((EmitIssue::AsyncComponent, span));
    }
    let params_span = self::span(params.span);
    let mut component = Component {
        name,
        span,
        form: Form::Initializer,
        head: Span::new(span.start, params_span.start),
        params,
        tail: Span::new(
            params_span.end,
            host_body.map_or(region.span.start, |body| body.span.start),
        ),
        region,
        body: host_body,
        expression,
        props: Props::None,
    };
    // Setup statements, without the host body's final `return`.
    let setup = match (host_body, &region.node) {
        (Some(body), _) => body
            .statements
            .split_last()
            .map_or(&[][..], |(_, rest)| rest),
        (None, Node::CodeBlock(block)) => block.statements.as_slice(),
        (None, _) => &[],
    };
    let mut returns = Returns(None);
    for statement in setup {
        returns.visit_statement(statement);
    }
    if let Some(early) = returns.0 {
        return Err((EmitIssue::EarlyReturn, early));
    }
    component.props = props::props(module, &component)?;
    Ok(Some(component))
}

/// The region a host body returns as its final statement, or an arrow's
/// expression body, parentheses aside.
fn returned_region<'a>(
    module: &'a TsrxModule<'a>,
    body: &'a FunctionBody<'a>,
    expression: bool,
) -> Option<&'a Region<'a>> {
    let mut value = match body.statements.last()? {
        Statement::ReturnStatement(statement) if !expression => statement.argument.as_ref()?,
        Statement::ExpressionStatement(statement) if expression => &statement.expression,
        _ => return None,
    };
    while let Expression::ParenthesizedExpression(inner) = value {
        value = &inner.expression;
    }
    module.expression_region(value)
}

/// The first `return` of setup statements, nested functions and classes aside.
struct Returns(Option<Span>);

impl<'a> Visit<'a> for Returns {
    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        self.0.get_or_insert(span(it.span));
    }

    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, _: &Class<'a>) {}
}

fn span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}
