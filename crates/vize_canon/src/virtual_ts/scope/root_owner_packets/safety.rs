//! Conservative relocation proof, activated only for real root EXPECT owners.

use std::cell::OnceCell;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Expression, ObjectPropertyKind, PropertyKind, Statement,
    VariableDeclarationKind,
};
use oxc_ast_visit::{Visit, walk::walk_expression};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashMap, FxHashSet, String, profile};
use vize_croquis::{BindingType, croquis::PassedProp};

use crate::virtual_ts::script_facts::ScriptBindings;

/// Const references can still be narrowed by a render-time property mutation
/// or assertion call. Never move across such a frontier. Handler/function
/// bodies are declarations, not render-time invocations, and remain isolated.
pub(super) fn stable_render_flow(root: &vize_relief::RootNode<'_>) -> bool {
    profile!(
        "canon.virtual_ts.root_expect.render_flow",
        stable_children(&root.children)
    )
}

fn stable_children(children: &[vize_relief::TemplateChildNode<'_>]) -> bool {
    use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};
    children.iter().all(|child| match child {
        TemplateChildNode::Interpolation(node) => stable_expression(&node.content),
        TemplateChildNode::Element(element) => {
            element.props.iter().all(|prop| {
                let PropNode::Directive(directive) = prop else {
                    return true;
                };
                if matches!(directive.name, "on" | "slot") {
                    return true;
                }
                if directive.name == "for" {
                    return false;
                }
                let argument = directive.arg.as_ref().is_none_or(|arg| {
                    matches!(arg, ExpressionNode::Simple(arg) if arg.is_static)
                        || stable_expression(arg)
                });
                argument && directive.exp.as_ref().is_none_or(stable_expression)
            }) && stable_children(&element.children)
        }
        TemplateChildNode::If(node) => node.branches.iter().all(|branch| {
            branch.condition.as_ref().is_none_or(stable_expression)
                && stable_children(&branch.children)
        }),
        // A loop has its own declaration/CFA environment; don't move a root
        // packet across an unproved replay of that environment.
        TemplateChildNode::For(_) => false,
        _ => true,
    })
}

fn stable_expression(node: &vize_relief::ExpressionNode<'_>) -> bool {
    let vize_relief::ExpressionNode::Simple(simple) = node else {
        return false;
    };
    let Some(retained) = simple.js_ast.and_then(|ast| ast.as_expression()) else {
        return false;
    };
    if retained.raw != simple.content {
        return false;
    }
    let mut effects = RenderEffects(false);
    effects.visit_expression(retained.ast);
    !effects.0
}

struct RenderEffects(bool);
impl<'a> Visit<'a> for RenderEffects {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        match expression {
            Expression::AssignmentExpression(_)
            | Expression::UpdateExpression(_)
            | Expression::CallExpression(_)
            | Expression::NewExpression(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::AwaitExpression(_)
            | Expression::YieldExpression(_)
            | Expression::ClassExpression(_) => self.0 = true,
            Expression::UnaryExpression(value)
                if value.operator == oxc_syntax::operator::UnaryOperator::Delete =>
            {
                self.0 = true;
            }
            Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_) => {}
            _ if !self.0 => walk_expression(self, expression),
            _ => {}
        }
    }
}

/// The cache is generation-local, and its key includes the lexical scope and
/// the exact authored expression identity. Ordinary generation never creates
/// ASTs or owned names through this helper.
pub(super) struct RelocationSafety<'a> {
    script: Option<&'a str>,
    bindings: ScriptBindings<'a>,
    initialized: OnceCell<FxHashSet<String>>,
    values: FxHashMap<(u32, u32, u32), bool>,
}

impl<'a> RelocationSafety<'a> {
    pub(super) fn new(script: Option<&'a str>, bindings: ScriptBindings<'a>) -> Self {
        Self {
            script,
            bindings,
            initialized: OnceCell::new(),
            values: FxHashMap::default(),
        }
    }

    pub(super) fn initialized_const(&self, name: &str) -> bool {
        matches!(
            self.bindings.binding_type(name),
            Some(BindingType::SetupConst | BindingType::LiteralConst)
        ) && self
            .initialized
            .get_or_init(|| initialized_consts(self.script))
            .contains(name)
    }

    pub(super) fn allows(&mut self, scope: u32, prop: &PassedProp) -> bool {
        let key = (scope, prop.start, prop.end);
        if let Some(&allowed) = self.values.get(&key) {
            return allowed;
        }
        let allowed = prop.value.as_ref().is_some_and(|value| {
            let allocator = Allocator::new();
            profile!("canon.virtual_ts.root_expect.parse_candidate", {
                Parser::new(&allocator, value.as_str(), SourceType::ts()).parse_expression()
            })
            .is_ok_and(|expression| match &expression {
                Expression::ArrowFunctionExpression(arrow) => pure_arrow(arrow),
                _ => pure_expression(&expression, &|name| self.initialized_const(name)),
            })
        });
        self.values.insert(key, allowed);
        allowed
    }
}

fn pure_expression(expression: &Expression<'_>, identifier: &impl Fn(&str) -> bool) -> bool {
    match expression {
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::Identifier(value) => identifier(value.name.as_str()),
        Expression::ParenthesizedExpression(value) => {
            pure_expression(&value.expression, identifier)
        }
        // A type query can capture a flow-sensitive outer name even when the
        // runtime operand is pure. Those types have no relocation proof.
        Expression::TSNonNullExpression(value) => pure_expression(&value.expression, identifier),
        Expression::UnaryExpression(value) => {
            value.operator != oxc_syntax::operator::UnaryOperator::Delete
                && pure_expression(&value.argument, identifier)
        }
        Expression::ArrayExpression(array) => array.elements.iter().all(|element| {
            element
                .as_expression()
                .is_some_and(|value| pure_expression(value, identifier))
        }),
        Expression::ObjectExpression(object) => object.properties.iter().all(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            !property.computed
                && !property.method
                && property.kind == PropertyKind::Init
                && pure_expression(&property.value, identifier)
        }),
        // In particular, do not move a member read/getter, invocation,
        // assignment/update, spread, or contextual callback body.
        _ => false,
    }
}

fn pure_arrow(arrow: &oxc_ast::ast::ArrowFunctionExpression<'_>) -> bool {
    if arrow.r#async
        || !arrow.expression
        || arrow.params.rest.is_some()
        || arrow.type_parameters.is_some()
        || arrow.return_type.is_some()
    {
        return false;
    }
    if arrow.params.items.iter().any(|parameter| {
        !matches!(parameter.pattern, BindingPattern::BindingIdentifier(_))
            || parameter.initializer.is_some()
            || parameter.type_annotation.is_some()
            || !parameter.decorators.is_empty()
    }) {
        return false;
    }
    let [Statement::ExpressionStatement(body)] = arrow.body.statements.as_slice() else {
        return false;
    };
    pure_expression(&body.expression, &|name| {
        !matches!(name, "arguments" | "this" | "super") && arrow.params.items.iter().any(|parameter| {
            matches!(&parameter.pattern, BindingPattern::BindingIdentifier(binding) if binding.name == name)
        })
    })
}

fn initialized_consts(script: Option<&str>) -> FxHashSet<String> {
    let Some(script) = script else {
        return FxHashSet::default();
    };
    let allocator = Allocator::new();
    let parsed = profile!("canon.virtual_ts.root_expect.parse_setup", {
        Parser::new(&allocator, script, SourceType::ts()).parse()
    });
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return FxHashSet::default();
    }
    parsed
        .program
        .body
        .iter()
        .filter_map(|statement| {
            let Statement::VariableDeclaration(declaration) = statement else {
                return None;
            };
            (declaration.kind == VariableDeclarationKind::Const && !declaration.declare)
                .then_some(&declaration.declarations)
        })
        .flatten()
        .filter_map(|declarator| {
            let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                return None;
            };
            declarator
                .init
                .as_ref()
                .map(|_| String::from(identifier.name.as_str()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    #[test]
    fn cached_proof_keeps_effects_captures_and_uninitialized_reads_in_place() {
        let script = "const stable = 1; const noop = () => {}; let mutable = 1; declare const declared: number; const café = 2;";
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();
        crate::virtual_ts::script_facts::with_bindings(&summary, |bindings| {
            let mut safety = RelocationSafety::new(Some(script), bindings);
            assert!(safety.values.is_empty());
            assert!(safety.initialized.get().is_none());
            for (index, (value, allowed)) in [
                ("{ role: null, tags: [] }", true),
                ("stable", true),
                ("café", true),
                ("[stable, { label: '😀' }]", true),
                ("(field) => !!field", true),
                ("(field) => [field, { value: !field }]", true),
                ("mutable", false),
                ("declared", false),
                ("missing", false),
                ("stable.value", false),
                ("noop()", false),
                ("mutable = 1", false),
                ("mutable++", false),
                ("{ get value() { return stable } }", false),
                ("{ [stable]: 1 }", false),
                ("[...stable]", false),
                ("(field) => stable", false),
                ("(field = stable) => field", false),
                ("({field}) => field", false),
                ("(...field) => field", false),
                ("async (field) => field", false),
                ("(field) => field.value", false),
                ("(field) => noop(field)", false),
                ("(field) => field++", false),
                ("(field) => () => field", false),
                ("(field) => this", false),
                ("(field: typeof stable.value) => !!field", false),
                ("(field: number) => !!field", false),
                ("(field): typeof stable.value => field", false),
                ("(field) => field as typeof stable.value", false),
                ("(field) => field satisfies typeof stable.value", false),
                ("(field) => <typeof stable.value>field", false),
                ("stable as typeof stable.value", false),
                ("stable satisfies typeof stable.value", false),
                ("<typeof stable.value>stable", false),
            ]
            .into_iter()
            .enumerate()
            {
                let prop = PassedProp {
                    name: "value".into(),
                    name_is_dynamic: false,
                    value: Some(value.into()),
                    start: index as u32,
                    end: index as u32 + 1,
                    is_dynamic: true,
                };
                assert_eq!(safety.allows(3, &prop), allowed, "{value}");
                let cached = safety.values.len();
                assert_eq!(safety.allows(3, &prop), allowed, "cached {value}");
                assert_eq!(safety.values.len(), cached);
                assert_eq!(
                    safety.allows(4, &prop),
                    allowed,
                    "another lexical scope {value}"
                );
                assert_eq!(safety.values.len(), cached + 1);
            }
        });
    }
}
