//! TSRX components of the Vue profile, read from the host program.
//!
//! A component is a function whose body is a statement container (`@{ … }`),
//! or that returns markup (`{ setup; return <markup>; }`, `() => <markup>`),
//! and whose static name starts with an uppercase ASCII letter: a function
//! declaration, or a `const`/`let` declarator initialized with an arrow or a
//! function expression, optionally exported. Setup returns nothing but the
//! render: an earlier `return` would decide the output once, at setup, and is
//! refused (`EarlyReturn`). Its props follow the pinned
//! reference (`vue-jsx-vapor` macros): the keys of a destructured first
//! parameter, which are read reactively through `__props`, or every
//! `props.name` read of an identifier first parameter.

use alloc::vec::Vec;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingPattern, Class, Declaration, Expression, FormalParameters,
    Function, FunctionBody, PropertyKey, ReturnStatement, Statement, VariableDeclaration,
};
use oxc_ast_visit::Visit;
use oxc_syntax::scope::ScopeFlags;
use vize_l0::Span;
use vize_l1::markup::tsrx::{Node, Region, RegionSite, TsrxModule};

use super::EmitIssue;
use super::names::Names;

/// One component whose body region is emitted.
pub(super) struct Component<'a> {
    pub name: &'a str,
    /// The host range the definition replaces.
    pub span: Span,
    /// A function declaration, which becomes a `const` declaration.
    pub declaration: bool,
    /// From the function start through the parameter list's `(`.
    pub head: Span,
    pub params: &'a FormalParameters<'a>,
    /// From the parameter list's `)` through the body: the `@` of a `@{ … }`
    /// region, or the start of a host body.
    pub tail: Span,
    pub region: &'a Region<'a>,
    /// The host body whose return value is the region, `None` for `@{ … }`.
    pub body: Option<&'a FunctionBody<'a>>,
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

/// How a component reads its props.
pub(super) enum Props<'a> {
    None,
    /// `({ a, b }) => …`: the keys, read as `__props.key`.
    Destructured {
        param: Span,
        keys: Vec<&'a str>,
    },
    /// `(props) => …`: every `props.name` read.
    Identifier {
        keys: Vec<&'a str>,
    },
}

impl Props<'_> {
    pub(super) fn keys(&self) -> &[&str] {
        match self {
            Self::None => &[],
            Self::Destructured { keys, .. } | Self::Identifier { keys, .. } => keys,
        }
    }
}

/// Every top-level component of the module, in source order. A body region
/// that is not a component body is refused.
pub(super) fn collect<'a>(
    module: &'a TsrxModule<'a>,
) -> Result<Vec<Component<'a>>, (EmitIssue, Span)> {
    let mut found = Vec::new();
    for statement in &module.program().body {
        match statement {
            Statement::FunctionDeclaration(function) => {
                declaration(module, function, &mut found)?;
            }
            Statement::VariableDeclaration(variables) => {
                declarators(module, variables, &mut found)?;
            }
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::FunctionDeclaration(function)) => {
                    declaration(module, function, &mut found)?;
                }
                Some(Declaration::VariableDeclaration(variables)) => {
                    declarators(module, variables, &mut found)?;
                }
                _ => {}
            },
            _ => {}
        }
    }
    for region in module.regions() {
        if !found
            .iter()
            .any(|component| core::ptr::eq(component.region, region))
        {
            return Err((EmitIssue::OutsideComponent, region.span));
        }
    }
    Ok(found)
}

fn declaration<'a>(
    module: &'a TsrxModule<'a>,
    function: &'a Function<'a>,
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
        found.push(Component {
            declaration: true,
            ..component
        });
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
        declaration: false,
        head: Span::new(span.start, params_span.start),
        params,
        tail: Span::new(
            params_span.end,
            host_body.map_or(region.span.start, |body| body.span.start),
        ),
        region,
        body: host_body,
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
    component.props = props(module.program().source_text, &component)?;
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

fn props<'a>(source: &'a str, component: &Component<'a>) -> Result<Props<'a>, (EmitIssue, Span)> {
    let params = component.params;
    let refuse = Err((EmitIssue::Parameters, span(params.span)));
    if params.rest.is_some() || params.items.len() > 1 {
        return refuse;
    }
    let Some(param) = params.items.first() else {
        return Ok(Props::None);
    };
    if param.initializer.is_some() || !param.decorators.is_empty() {
        return refuse;
    }
    let names = Names::of_component(source, component);
    match &param.pattern {
        BindingPattern::BindingIdentifier(id) => {
            let name = id.name.as_str();
            if let Some(use_) = names.value_reads(name).next() {
                return Err((EmitIssue::PropsValue, use_));
            }
            Ok(Props::Identifier {
                keys: names.member_keys(name),
            })
        }
        BindingPattern::ObjectPattern(pattern) => {
            if pattern.rest.is_some() {
                return refuse;
            }
            let mut keys = Vec::with_capacity(pattern.properties.len());
            for property in &pattern.properties {
                let (PropertyKey::StaticIdentifier(key), BindingPattern::BindingIdentifier(local)) =
                    (&property.key, &property.value)
                else {
                    return refuse;
                };
                if property.computed || key.name != local.name {
                    return refuse;
                }
                keys.push(key.name.as_str());
            }
            for key in &keys {
                if let Some(redeclared) = names.declarations(key).next() {
                    return Err((EmitIssue::ShadowedProp, redeclared));
                }
                if let Some(tag) = names.tags(key).next() {
                    return Err((EmitIssue::PropsValue, tag));
                }
            }
            Ok(Props::Destructured {
                param: span(param.span),
                keys,
            })
        }
        _ => refuse,
    }
}

fn span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}
