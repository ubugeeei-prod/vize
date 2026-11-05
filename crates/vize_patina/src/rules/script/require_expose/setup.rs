//! Setup `expose()` calls and render-function returns.

use super::Kind;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingPattern, Expression, FormalParameters, Function,
    IdentifierReference, ObjectPattern, Statement,
};
use oxc_ast_visit::Visit;
use oxc_semantic::{AstNodes, Semantic};
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use oxc_syntax::scope::ScopeFlags;
use oxc_syntax::symbol::SymbolId;
use vize_l0::FxHashSet;

pub(super) fn is_exempt<'a>(value: &Expression<'a>, semantic: &Semantic<'a>) -> bool {
    match value.without_parentheses() {
        Expression::FunctionExpression(function) => {
            calls_expose(&function.params, semantic)
                || returns_render(function.body.as_deref(), semantic)
        }
        Expression::ArrowFunctionExpression(arrow) => {
            calls_expose(&arrow.params, semantic) || arrow_returns_render(arrow, semantic)
        }
        _ => false,
    }
}

pub(super) fn is_object_argument(argument: &Argument<'_>) -> bool {
    argument.as_expression().is_some_and(|expression| {
        matches!(skip_wrappers(expression), Expression::ObjectExpression(_))
    })
}

pub(super) fn skip_wrappers<'b, 'a>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => skip_wrappers(&inner.expression),
        Expression::TSAsExpression(inner) => skip_wrappers(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => skip_wrappers(&inner.expression),
        Expression::TSNonNullExpression(inner) => skip_wrappers(&inner.expression),
        other => other,
    }
}

/// Upstream's `getVueComponentDefinitionType`, without treating a mixin as a component.
pub(super) fn classify<'a>(semantic: &Semantic<'a>, callee: &Expression<'a>) -> Option<Kind> {
    match callee.without_parentheses() {
        Expression::Identifier(identifier) => match identifier.name.as_str() {
            "defineNuxtComponent" => Some(Kind::Component),
            "component" | "createApp" | "defineComponent" if vue_global(semantic, identifier) => {
                Some(Kind::Component)
            }
            _ => None,
        },
        callee => {
            let member = callee.as_member_expression()?;
            let Expression::Identifier(object) = skip_wrappers(member.object()) else {
                return None;
            };
            match member.static_property_name() {
                Some("mixin") => Some(Kind::Mixin),
                Some("component") => Some(Kind::Component),
                Some("extend") if object.name == "Vue" => Some(Kind::Component),
                _ => None,
            }
        }
    }
}

fn calls_expose<'a>(params: &FormalParameters<'a>, semantic: &Semantic<'a>) -> bool {
    // A rest or defaulted context parameter is not tracked.
    if params.rest.is_some() && params.items.len() < 2 {
        return false;
    }
    let Some(param) = params.items.get(1) else {
        return false;
    };
    if param.initializer.is_some() {
        return false;
    }
    match &param.pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            member_calls(identifier.symbol_id.get(), semantic)
        }
        BindingPattern::ObjectPattern(pattern) => direct_calls(expose_symbol(pattern), semantic),
        BindingPattern::ArrayPattern(_) | BindingPattern::AssignmentPattern(_) => false,
    }
}

fn expose_symbol(pattern: &ObjectPattern<'_>) -> Option<SymbolId> {
    let property = pattern.properties.iter().find(|property| {
        !property.computed && property.key.static_name().as_deref() == Some("expose")
    })?;
    let BindingPattern::BindingIdentifier(identifier) = &property.value else {
        return None;
    };
    identifier.symbol_id.get()
}

fn direct_calls(symbol: Option<SymbolId>, semantic: &Semantic<'_>) -> bool {
    let Some(symbol) = symbol else {
        return false;
    };
    semantic.symbol_references(symbol).any(|reference| {
        if !reference.is_read() {
            return false;
        }
        let nodes = semantic.nodes();
        let AstKind::CallExpression(call) = nodes.parent_kind(reference.node_id()) else {
            return false;
        };
        let AstKind::IdentifierReference(identifier) = nodes.get_node(reference.node_id()).kind()
        else {
            return false;
        };
        call.callee.span() == identifier.span
    })
}

fn member_calls(symbol: Option<SymbolId>, semantic: &Semantic<'_>) -> bool {
    let Some(symbol) = symbol else {
        return false;
    };
    semantic.symbol_references(symbol).any(|reference| {
        if !reference.is_read() {
            return false;
        }
        let nodes = semantic.nodes();
        let parent_id = nodes.parent_id(reference.node_id());
        let AstKind::StaticMemberExpression(member) = nodes.kind(parent_id) else {
            return false;
        };
        member.property.name == "expose" && invoked(nodes, parent_id, member.span)
    })
}

fn invoked(nodes: &AstNodes<'_>, node_id: NodeId, span: Span) -> bool {
    let parent_id = nodes.parent_id(node_id);
    match nodes.kind(parent_id) {
        AstKind::CallExpression(call) => call.callee.span() == span,
        AstKind::ChainExpression(chain) => match nodes.parent_kind(parent_id) {
            AstKind::CallExpression(call) => call.callee.span() == chain.span,
            _ => false,
        },
        _ => false,
    }
}

fn arrow_returns_render(arrow: &ArrowFunctionExpression<'_>, semantic: &Semantic<'_>) -> bool {
    if arrow.expression {
        let Some(Statement::ExpressionStatement(statement)) = arrow.body.statements.first() else {
            return false;
        };
        return is_function(&statement.expression, semantic, &mut FxHashSet::default());
    }
    returns_render(Some(&arrow.body), semantic)
}

fn returns_render(body: Option<&oxc_ast::ast::FunctionBody<'_>>, semantic: &Semantic<'_>) -> bool {
    let Some(body) = body else {
        return false;
    };
    let mut probe = RenderProbe {
        semantic,
        found: false,
    };
    probe.visit_function_body(body);
    probe.found
}

struct RenderProbe<'s, 'a> {
    semantic: &'s Semantic<'a>,
    found: bool,
}

impl<'a> Visit<'a> for RenderProbe<'_, 'a> {
    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_return_statement(&mut self, it: &oxc_ast::ast::ReturnStatement<'a>) {
        if let Some(argument) = &it.argument
            && is_function(argument, self.semantic, &mut FxHashSet::default())
        {
            self.found = true;
        }
    }
}

fn is_function<'a>(
    expression: &Expression<'a>,
    semantic: &Semantic<'a>,
    seen: &mut FxHashSet<SymbolId>,
) -> bool {
    match expression.without_parentheses() {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::Identifier(identifier) => {
            let Some(symbol) = identifier
                .reference_id
                .get()
                .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())
            else {
                return false;
            };
            if !seen.insert(symbol) {
                return false;
            }
            match semantic.symbol_declaration(symbol).kind() {
                AstKind::Function(_) => true,
                AstKind::VariableDeclarator(declarator) => declarator
                    .init
                    .as_ref()
                    .is_some_and(|init| is_function(init, semantic, seen)),
                _ => false,
            }
        }
        _ => false,
    }
}

fn vue_global(semantic: &Semantic<'_>, identifier: &IdentifierReference<'_>) -> bool {
    let scoping = semantic.scoping();
    let Some(symbol) = identifier
        .reference_id
        .get()
        .and_then(|reference| scoping.get_reference(reference).symbol_id())
    else {
        return true;
    };
    if !scoping.symbol_redeclarations(symbol).is_empty() {
        return true;
    }
    let declaration = semantic.symbol_declaration(symbol);
    let nodes = semantic.nodes();
    match declaration.kind() {
        AstKind::ImportSpecifier(specifier) if specifier.imported.is_identifier() => {
            matches!(
                nodes.parent_kind(declaration.id()),
                AstKind::ImportDeclaration(import)
                    if matches!(import.source.value.as_str(), "vue" | "@vue/composition-api")
            )
        }
        AstKind::VariableDeclarator(declarator)
            if matches!(declarator.id, BindingPattern::ObjectPattern(_)) =>
        {
            match declarator
                .init
                .as_ref()
                .map(Expression::without_parentheses)
            {
                Some(Expression::CallExpression(call)) if matches!(&call.callee, Expression::Identifier(id) if id.name == "require") =>
                {
                    matches!(
                        call.arguments.first(),
                        Some(Argument::StringLiteral(source))
                            if matches!(source.value.as_str(), "vue" | "@vue/composition-api")
                    )
                }
                _ => true,
            }
        }
        AstKind::Function(_) | AstKind::Class(_) | AstKind::CatchParameter(_) => false,
        _ => true,
    }
}
