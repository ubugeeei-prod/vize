//! `<component :is="expr">` whose `expr` is not a component identifier.
//!
//! Croquis records such a usage under a synthetic name (see
//! `vize_croquis::drawer::dynamic_component_alias`); the template scope
//! declares that name as a `const` initialized with the authored expression,
//! so the usage's props, listeners and slots are checked against
//! `typeof <alias>` exactly like a named component's. A union of constructors
//! (`cond ? Foo : Bar`) then types each prop from the members that declare it,
//! which is how `vue-tsc` checks the same element.

use vize_carton::{FxHashMap, FxHashSet, String, append};
use vize_croquis::{Croquis, ScopeKind, drawer::is_dynamic_component_alias};
use vize_relief::{ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode};

mod generic_index;
mod homogeneous_index;

/// A named value of the built-in `component`, not an authored registry tag.
/// Match the existing Croquis retained-AST authority without reparsing.
pub(crate) fn is_owned_named_dynamic_component(
    template_ast: Option<&RootNode<'_>>,
    usage: &vize_croquis::croquis::ComponentUsage,
) -> bool {
    owned_named_dynamic_component_ast(template_ast, usage).is_some()
}

pub(super) fn owned_named_dynamic_component_ast<'a>(
    template_ast: Option<&'a RootNode<'a>>,
    usage: &vize_croquis::croquis::ComponentUsage,
) -> Option<&'a oxc_ast::ast::Expression<'a>> {
    let ExpressionNode::Simple(expression) = is_expression_node(template_ast?, usage.start)? else {
        return None;
    };
    let js = expression.js_ast?.as_expression()?;
    (js.raw == expression.content
        && (usage.name == expression.content.trim()
            || usage.name == vize_croquis::drawer::dynamic_component_alias(usage.start))
        && matches!(
            js.ast,
            oxc_ast::ast::Expression::Identifier(_)
                | oxc_ast::ast::Expression::StaticMemberExpression(_)
        ))
    .then_some(js.ast)
}

/// Prove the usage is the generated alias of its authored `<component :is>`.
/// Reserved-looking authored static tags must still receive registry checks.
pub(crate) fn is_owned_dynamic_component_alias(
    template_ast: Option<&RootNode<'_>>,
    usage: &vize_croquis::croquis::ComponentUsage,
) -> bool {
    is_dynamic_component_alias(usage.name.as_str())
        && usage.name == vize_croquis::drawer::dynamic_component_alias(usage.start)
        && template_ast.is_some_and(|root| {
            let Some(ExpressionNode::Simple(expression)) = is_expression_node(root, usage.start)
            else {
                return false;
            };
            expression
                .js_ast
                .and_then(|js| js.as_expression())
                .is_some_and(|js| js.raw == expression.content)
        })
}

/// Every alias, declared in the scope its `:is` is evaluated in. That scope
/// binds every name the alias reads, so a slot, loop or pattern arm declares
/// its aliases at the top of its body, in the template pass and again in the
/// component-props pass; the template root declares the rest.
#[derive(Default)]
pub(crate) struct DynamicComponentAliases {
    root: Vec<AliasDeclaration>,
    scoped: FxHashMap<u32, Vec<AliasDeclaration>>,
    scoped_starts: FxHashSet<u32>,
}

struct AliasDeclaration {
    start: u32,
    comment: &'static str,
    statement: String,
}

impl DynamicComponentAliases {
    pub(super) fn collect(
        summary: &Croquis,
        template_ast: Option<&RootNode<'_>>,
        generic: bool,
    ) -> Self {
        let mut aliases = Self::default();
        let Some(root) = template_ast else {
            return aliases;
        };
        for usage in vize_croquis::facts::component_usage_slice(summary) {
            if !is_dynamic_component_alias(usage.name.as_str()) {
                continue;
            }
            let Some(expression) = is_expression(root, usage.start) else {
                continue;
            };
            // The authored `v-bind` statement owns the expression's own
            // diagnostics; the alias only has to carry its type.
            // The original Program proves equal declared entry types. Select an
            // actual constant-key property to retain ordinary literal widening.
            let annotation = homogeneous_index::value(summary, root, usage)
                .map(|value| vize_carton::cstr!(": {value}"))
                .unwrap_or_default();
            // Only a generic setup can defer an indexed entry type.
            let boxed = generic.then(|| generic_index::value(root, usage)).flatten();
            let expression = boxed.as_deref().unwrap_or(expression).trim();
            let declaration = match &usage.vif_guard {
                Some(guard) => AliasDeclaration {
                    start: usage.start,
                    comment: "// @ts-ignore Inference-only alias; authored v-if and v-bind statements own diagnostics.",
                    statement: vize_carton::cstr!(
                        "const {}{annotation} = ({guard}) ? ({expression}) : (undefined as never);",
                        usage.name
                    ),
                },
                None => AliasDeclaration {
                    start: usage.start,
                    comment: "// @ts-ignore Inference-only alias; the authored v-bind statement owns diagnostics.",
                    statement: vize_carton::cstr!(
                        "const {}{annotation} = ({expression});",
                        usage.name
                    ),
                },
            };
            let scope = summary
                .scopes
                .get_scope(usage.scope_id)
                .filter(|scope| {
                    matches!(
                        scope.kind,
                        ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen
                    )
                })
                .map(|scope| scope.id.as_u32());
            match scope {
                Some(scope) => {
                    aliases.scoped_starts.insert(usage.start);
                    aliases.scoped.entry(scope).or_default().push(declaration);
                }
                None => aliases.root.push(declaration),
            }
        }
        aliases
    }

    /// Whether the usage at `start` resolves through an alias that only its
    /// own scope can see.
    pub(super) fn is_scoped(&self, start: u32) -> bool {
        self.scoped_starts.contains(&start)
    }

    pub(super) fn emit_root(&self, ts: &mut String) {
        for declaration in &self.root {
            declaration.push(ts, "  ");
        }
    }

    /// Declare the aliases of `scope`, each followed by `then` with its
    /// usage start.
    pub(super) fn emit_scope(
        &self,
        ts: &mut String,
        scope: u32,
        indent: &str,
        mut then: impl FnMut(&mut String, u32),
    ) {
        for declaration in self.scoped.get(&scope).into_iter().flatten() {
            declaration.push(ts, indent);
            then(ts, declaration.start);
        }
    }
}

impl AliasDeclaration {
    fn push(&self, ts: &mut String, indent: &str) {
        append!(
            *ts,
            "{indent}{}\n{indent}{}\n",
            self.comment,
            self.statement
        );
    }
}

/// The `:is` expression of the `<component>` element starting at `start`.
fn is_expression<'a>(root: &'a RootNode<'a>, start: u32) -> Option<&'a str> {
    match is_expression_node(root, start)? {
        ExpressionNode::Simple(simple) => Some(simple.content),
        ExpressionNode::Compound(compound) => Some(compound.loc.span.slice(root.source)),
    }
}

fn is_expression_node<'a>(root: &'a RootNode<'a>, start: u32) -> Option<&'a ExpressionNode<'a>> {
    let mut found = None;
    for child in root.children.iter() {
        visit(child, start, &mut found);
        if found.is_some() {
            break;
        }
    }
    found
}

fn visit<'a>(
    node: &'a TemplateChildNode<'a>,
    start: u32,
    found: &mut Option<&'a ExpressionNode<'a>>,
) {
    match node {
        TemplateChildNode::Element(element) => {
            if element.loc.span.start == start {
                *found = (element.tag == "component")
                    .then(|| is_directive_expression(element))
                    .flatten();
                return;
            }
            for child in element.children.iter() {
                visit(child, start, found);
                if found.is_some() {
                    return;
                }
            }
        }
        TemplateChildNode::If(node) => {
            for branch in node.branches.iter() {
                for child in branch.children.iter() {
                    visit(child, start, found);
                    if found.is_some() {
                        return;
                    }
                }
            }
        }
        TemplateChildNode::For(node) => {
            for child in node.children.iter() {
                visit(child, start, found);
                if found.is_some() {
                    return;
                }
            }
        }
        _ => {}
    }
}

fn is_directive_expression<'a>(element: &'a ElementNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    element.props.iter().find_map(|prop| {
        let PropNode::Directive(directive) = prop else {
            return None;
        };
        if directive.name != "bind"
            || !matches!(
                directive.arg.as_ref(),
                Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == "is"
            )
        {
            return None;
        }
        directive.exp.as_ref()
    })
}
