//! Receiver identity for the removed instance event API.
//!
//! A call is reported only when its receiver is provably a Vue instance:
//! `this` (and `const vm = this` aliases), the public `$root` / `$parent`
//! relations of a proven instance, and module-local bindings constructed with
//! `new Vue(...)` / `new (Vue.extend(...))(...)` that never escape. Imports,
//! injected or plugin properties, call results and unbound names are not
//! proven: they are often a migration shim (`{ $on: (...a) => emitter.on(...a) }`)
//! and stay silent.
//!
//! A file that authors a `$on` / `$off` / `$once` definition (object key, class
//! member, member write, `defineProperty`-style string argument), or hands an
//! instance to an external call as its first argument (`Object.assign(this, shim)`),
//! may install a foreign method on any instance; that method name is not
//! reported anywhere in the file.

use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentOperator, BindingPattern, CallExpression, ClassElement, Expression,
        IdentifierReference, MemberExpression, NewExpression, ObjectProperty, Program,
        SimpleAssignmentTarget,
    },
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_call_expression, walk_class_element, walk_new_expression, walk_object_property,
        walk_simple_assignment_target,
    },
};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::GetSpan;
use oxc_syntax::{node::NodeId, symbol::SymbolId};

use super::REMOVED_EVENT_METHODS;

/// Bounds alias chains such as `const a = this; const b = a; ...`.
const MAX_DEPTH: u8 = 8;

pub(super) struct Receivers<'a> {
    semantic: Semantic<'a>,
    /// Indexed like [`REMOVED_EVENT_METHODS`].
    shadowed: [bool; 3],
}

impl<'a> Receivers<'a> {
    pub(super) fn new(program: &'a Program<'a>) -> Self {
        let semantic = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(program)
            .semantic;
        let mut scan = DefinitionScan {
            semantic: &semantic,
            shadowed: [false; 3],
        };
        scan.visit_program(program);
        let shadowed = scan.shadowed;
        Self { semantic, shadowed }
    }

    pub(super) fn reports(&self, method: &str, receiver: &Expression<'a>) -> bool {
        method_index(method)
            .and_then(|index| self.shadowed.get(index))
            .is_some_and(|shadowed| !shadowed)
            && self.instance(receiver, 0)
    }

    fn instance(&self, expression: &Expression<'a>, depth: u8) -> bool {
        if depth > MAX_DEPTH {
            return false;
        }
        match expression.get_inner_expression() {
            Expression::ThisExpression(_) => true,
            Expression::Identifier(identifier) => self
                .local_values(identifier, true)
                .is_some_and(|values| values.iter().all(|value| self.instance(value, depth + 1))),
            Expression::NewExpression(new) => self.constructor(&new.callee, depth + 1),
            Expression::ChainExpression(chain) => chain
                .expression
                .as_member_expression()
                .is_some_and(|member| self.relation(member, depth)),
            Expression::CallExpression(call) => mounted(call).is_some_and(|receiver| {
                // `vm.$mount()` returns `vm`.
                self.instance(receiver, depth + 1)
            }),
            expression => expression
                .as_member_expression()
                .is_some_and(|member| self.relation(member, depth)),
        }
    }

    /// `$root` / `$parent` are public instances of a proven instance.
    fn relation(&self, member: &MemberExpression<'a>, depth: u8) -> bool {
        matches!(member.static_property_name(), Some("$root" | "$parent"))
            && self.instance(member.object(), depth + 1)
    }

    fn constructor(&self, expression: &Expression<'a>, depth: u8) -> bool {
        if depth > MAX_DEPTH {
            return false;
        }
        match expression.get_inner_expression() {
            Expression::Identifier(identifier) => {
                self.vue_import(identifier)
                    || self.local_values(identifier, false).is_some_and(|values| {
                        values
                            .iter()
                            .all(|value| self.constructor(value, depth + 1))
                    })
            }
            Expression::CallExpression(call) => call
                .callee
                .get_inner_expression()
                .as_member_expression()
                .is_some_and(|member| {
                    member.static_property_name() == Some("extend")
                        && self.constructor(member.object(), depth + 1)
                }),
            _ => false,
        }
    }

    /// `Vue` imported from `vue`, or the unbound UMD global.
    fn vue_import(&self, identifier: &IdentifierReference<'a>) -> bool {
        if identifier.name != "Vue" {
            return false;
        }
        let Some(symbol) = self.symbol(identifier) else {
            return true;
        };
        let declaration = self.semantic.symbol_declaration(symbol);
        let imported = match declaration.kind() {
            AstKind::ImportDefaultSpecifier(_) => true,
            AstKind::ImportSpecifier(specifier) => specifier.imported.name() == "Vue",
            _ => false,
        };
        imported
            && matches!(
                self.semantic.nodes().parent_kind(declaration.id()),
                AstKind::ImportDeclaration(import) if import.source.value == "vue"
            )
    }

    fn symbol(&self, identifier: &IdentifierReference<'a>) -> Option<SymbolId> {
        let reference = identifier.reference_id.get()?;
        self.semantic.scoping().get_reference(reference).symbol_id()
    }

    /// Every value a plain local binding can hold: its initializer and each
    /// `=` assignment. `None` for imports, parameters, destructuring, other
    /// writes, or (when `contained`) any use other than a member read or
    /// `new` callee, since an escaped value may be mutated out of view.
    fn local_values(
        &self,
        identifier: &IdentifierReference<'a>,
        contained: bool,
    ) -> Option<Vec<&'a Expression<'a>>> {
        let symbol = self.symbol(identifier)?;
        let AstKind::VariableDeclarator(declarator) =
            self.semantic.symbol_declaration(symbol).kind()
        else {
            return None;
        };
        if !matches!(declarator.id, BindingPattern::BindingIdentifier(_)) {
            return None;
        }
        let mut values: Vec<&'a Expression<'a>> = declarator.init.iter().collect();
        let this_alias = values
            .iter()
            .all(|value| matches!(value.get_inner_expression(), Expression::ThisExpression(_)));
        let nodes = self.semantic.nodes();
        for reference in self.semantic.symbol_references(symbol) {
            let node = reference.node_id();
            let span = nodes.get_node(node).kind().span();
            if reference.is_write() {
                let AstKind::AssignmentExpression(assignment) = nodes.parent_kind(node) else {
                    return None;
                };
                if assignment.operator != AssignmentOperator::Assign
                    || assignment.left.span() != span
                {
                    return None;
                }
                values.push(&assignment.right);
            } else if contained && !this_alias && !self.contained_read(node) {
                return None;
            }
        }
        (!values.is_empty()).then_some(values)
    }

    fn contained_read(&self, node: NodeId) -> bool {
        let nodes = self.semantic.nodes();
        let span = nodes.get_node(node).kind().span();
        let parent = nodes.parent_node(node);
        let (object, key) = match parent.kind() {
            AstKind::StaticMemberExpression(member) => {
                (member.object.span(), Some(member.property.name.as_str()))
            }
            AstKind::ComputedMemberExpression(member) => (
                member.object.span(),
                member.static_property_name().map(|name| name.as_str()),
            ),
            AstKind::NewExpression(new) => return new.callee.span() == span,
            _ => return false,
        };
        // Writing an ordinary named key cannot replace an event method: authored
        // `$on` / `$off` / `$once` writes are covered by the definition scan.
        object == span
            && (!written(nodes.parent_kind(parent.id()), parent.kind().span())
                || key.is_some_and(|key| key != "__proto__"))
    }
}

fn method_index(method: &str) -> Option<usize> {
    REMOVED_EVENT_METHODS
        .iter()
        .position(|candidate| *candidate == method)
}

fn mounted<'b, 'a>(call: &'b CallExpression<'a>) -> Option<&'b Expression<'a>> {
    let member = call.callee.get_inner_expression().as_member_expression()?;
    (member.static_property_name() == Some("$mount")).then(|| member.object())
}

/// Whether a member at `span` is the target of a write in `parent`.
fn written(parent: AstKind<'_>, span: oxc_span::Span) -> bool {
    match parent {
        AstKind::AssignmentExpression(assignment) => assignment.left.span() == span,
        AstKind::UpdateExpression(_)
        | AstKind::ArrayAssignmentTarget(_)
        | AstKind::ObjectAssignmentTarget(_)
        | AstKind::AssignmentTargetWithDefault(_)
        | AstKind::AssignmentTargetPropertyProperty(_)
        | AstKind::AssignmentTargetRest(_)
        | AstKind::ForInStatement(_)
        | AstKind::ForOfStatement(_) => true,
        AstKind::UnaryExpression(unary) => unary.operator.is_delete(),
        _ => false,
    }
}

struct DefinitionScan<'s, 'a> {
    semantic: &'s Semantic<'a>,
    shadowed: [bool; 3],
}

impl DefinitionScan<'_, '_> {
    fn define(&mut self, name: Option<&str>) {
        if let Some(slot) = name
            .and_then(method_index)
            .and_then(|index| self.shadowed.get_mut(index))
        {
            *slot = true;
        }
    }

    fn arguments(&mut self, callee: &Expression<'_>, arguments: &[Argument<'_>]) {
        for argument in arguments {
            if let Some(Expression::StringLiteral(literal)) = argument.as_expression() {
                self.define(Some(literal.value.as_str()));
            }
            if let Some(Expression::TemplateLiteral(literal)) = argument.as_expression() {
                self.define(literal.single_quasi().as_deref());
            }
        }
        // A merge-shaped call (`extend(this, options)`) may copy unseen methods
        // onto the instance; a named key (`defineProperty(this, "x", ...)`) is
        // covered by the literal scan above.
        let [first, second, ..] = arguments else {
            return;
        };
        let named = matches!(
            second.as_expression(),
            Some(Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
        );
        if !named
            && first
                .as_expression()
                .is_some_and(|first| self.this_like(first, 0))
            && !self.this_like_root(callee)
        {
            self.shadowed = [true; 3];
        }
    }

    fn this_like(&self, expression: &Expression<'_>, depth: u8) -> bool {
        match expression.get_inner_expression() {
            Expression::ThisExpression(_) => true,
            Expression::Identifier(identifier) if depth < MAX_DEPTH => {
                let Some(symbol) = identifier.reference_id.get().and_then(|reference| {
                    self.semantic.scoping().get_reference(reference).symbol_id()
                }) else {
                    return false;
                };
                matches!(
                    self.semantic.symbol_declaration(symbol).kind(),
                    AstKind::VariableDeclarator(declarator)
                        if matches!(declarator.id, BindingPattern::BindingIdentifier(_))
                            && declarator.init.as_ref().is_some_and(|init| self.this_like(init, depth + 1))
                )
            }
            _ => false,
        }
    }

    /// Methods reached through the instance (`this.$emit(..., this)`) are its own.
    fn this_like_root(&self, callee: &Expression<'_>) -> bool {
        let mut expression = callee.get_inner_expression();
        while let Some(member) = expression.as_member_expression() {
            expression = member.object().get_inner_expression();
        }
        self.this_like(expression, 0)
    }
}

impl<'a> Visit<'a> for DefinitionScan<'_, 'a> {
    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        self.define(it.key.static_name().as_deref());
        walk_object_property(self, it);
    }

    fn visit_class_element(&mut self, it: &ClassElement<'a>) {
        self.define(it.static_name().as_deref());
        walk_class_element(self, it);
    }

    fn visit_simple_assignment_target(&mut self, it: &SimpleAssignmentTarget<'a>) {
        if let Some(member) = it.as_member_expression() {
            self.define(member.static_property_name());
        }
        walk_simple_assignment_target(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        self.arguments(&it.callee, &it.arguments);
        walk_call_expression(self, it);
    }

    fn visit_new_expression(&mut self, it: &NewExpression<'a>) {
        self.arguments(&it.callee, &it.arguments);
        walk_new_expression(self, it);
    }
}
