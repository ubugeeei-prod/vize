//! Script reference facts with the typescript-eslint `no-unused-vars` and
//! ESLint `prefer-const` reference semantics.
//!
//! A read does not count as a use when it only feeds a write to the same
//! binding (`a += 1`, `x = foo(x)`), or when it sits inside the binding's own
//! declaration (`const a = () => a()`, `type Foo = Array<Foo>`). Parentheses
//! are transparent, as ESTree drops them.

use oxc_ast::{
    AstKind,
    ast::{
        ArrayAssignmentTarget, AssignmentTarget, AssignmentTargetMaybeDefault,
        AssignmentTargetProperty, ObjectAssignmentTarget,
    },
};
use oxc_semantic::{AstNodes, Scoping};
use oxc_span::{GetSpan, Span};
use oxc_syntax::{node::NodeId, operator::AssignmentOperator, scope::ScopeId, symbol::SymbolId};
use vize_l0::FxHashSet;

use super::classify::BindingInfo;

pub(super) struct Facts {
    /// Read by the script, as typescript-eslint counts reads.
    pub(super) used: bool,
    /// Any script reference at all, read or write.
    pub(super) any_reference: bool,
    pub(super) any_write: bool,
    /// The last write from the binding's own function scope.
    pub(super) last_local_write: Option<Span>,
}

struct Ref {
    span: Span,
    node: NodeId,
    scope: ScopeId,
    read: bool,
    write: bool,
}

pub(super) struct Usage<'s, 'a> {
    scoping: &'s Scoping,
    nodes: &'s AstNodes<'a>,
}

impl<'s, 'a> Usage<'s, 'a> {
    pub(super) fn new(scoping: &'s Scoping, nodes: &'s AstNodes<'a>) -> Self {
        Self { scoping, nodes }
    }

    fn references(&self, id: SymbolId) -> Vec<Ref> {
        let mut refs: Vec<Ref> = self
            .scoping
            .get_resolved_reference_ids(id)
            .iter()
            .map(|reference_id| {
                let reference = self.scoping.get_reference(*reference_id);
                let flags = reference.flags();
                Ref {
                    span: self.nodes.kind(reference.node_id()).span(),
                    node: reference.node_id(),
                    scope: reference.scope_id(),
                    read: flags.is_read() || flags.is_type() || flags.is_value_as_type(),
                    write: flags.is_write(),
                }
            })
            .collect();
        refs.sort_by_key(|reference| reference.span.start);
        refs
    }

    pub(super) fn facts(
        &self,
        id: SymbolId,
        info: &BindingInfo,
        for_return_targets: &FxHashSet<u32>,
    ) -> Facts {
        let refs = self.references(id);
        let home = self.variable_scope(self.scoping.symbol_scope_id(id));
        let mut used = info.for_return;
        let mut rhs = None;
        for reference in &refs {
            if used {
                break;
            }
            if for_return_targets.contains(&reference.span.start) {
                used = true;
                break;
            }
            let for_itself = self.read_for_itself(reference, rhs);
            rhs = self.rhs_node(reference, rhs, home);
            let own = info
                .self_spans
                .iter()
                .any(|span| contains(*span, reference.span));
            used = reference.read && !for_itself && !own;
        }
        let last_local_write = refs
            .iter()
            .filter(|reference| reference.write && self.variable_scope(reference.scope) == home)
            .map(|reference| reference.span)
            .next_back();
        Facts {
            used,
            any_reference: !refs.is_empty(),
            any_write: refs.iter().any(|reference| reference.write),
            last_local_write,
        }
    }

    /// Where ESLint `prefer-const` reports a `let`, or `None` to stay quiet.
    ///
    /// A binding with an initializer must never be written again. A binding
    /// without one must be written exactly once, by a plain `=` statement in
    /// its own block, and the report then points at that write. A read before
    /// the write moves the report to the declaration.
    pub(super) fn const_target(
        &self,
        id: SymbolId,
        info: &BindingInfo,
        ignore_read_before_assign: bool,
    ) -> Option<Span> {
        let refs = self.references(id);
        let mut writes = refs.iter().filter(|reference| reference.write);
        if info.has_init {
            return match writes.next() {
                Some(_) => None,
                None => Some(info.init_span.unwrap_or(self.scoping.symbol_span(id))),
            };
        }
        let writer = writes.next()?;
        if writes.next().is_some()
            || writer.scope != self.scoping.symbol_scope_id(id)
            || !self.becomes_declaration(writer.node, self.scoping.symbol_scope_id(id))
        {
            return None;
        }
        let read_before = refs
            .iter()
            .any(|reference| reference.read && reference.span.start < writer.span.start);
        if !read_before {
            return Some(writer.span);
        }
        (!ignore_read_before_assign).then(|| self.scoping.symbol_span(id))
    }

    fn variable_scope(&self, scope: ScopeId) -> ScopeId {
        self.scoping
            .scope_ancestors(scope)
            .find(|ancestor| self.scoping.scope_flags(*ancestor).is_var())
            .unwrap_or(scope)
    }

    fn parent(&self, node: NodeId) -> NodeId {
        let mut current = node;
        loop {
            let parent = self.nodes.parent_id(current);
            if parent == current
                || !matches!(self.nodes.kind(parent), AstKind::ParenthesizedExpression(_))
            {
                return parent;
            }
            current = parent;
        }
    }

    fn span(&self, node: NodeId) -> Span {
        self.nodes.kind(node).span()
    }

    /// The expression's value is discarded: a statement, or a non-final
    /// sequence member.
    fn unused_expression(&self, node: NodeId) -> bool {
        let span = self.span(node);
        let parent = self.parent(node);
        match self.nodes.kind(parent) {
            AstKind::ExpressionStatement(_) => true,
            AstKind::SequenceExpression(sequence) => {
                let last = sequence
                    .expressions
                    .last()
                    .map(|expression| expression.without_parentheses().span());
                last != Some(span) || self.unused_expression(parent)
            }
            _ => false,
        }
    }

    fn read_for_itself(&self, reference: &Ref, rhs: Option<Span>) -> bool {
        if !reference.read {
            return false;
        }
        let parent = self.parent(reference.node);
        let self_update = match self.nodes.kind(parent) {
            AstKind::AssignmentExpression(assignment) => {
                assignment.left.span() == reference.span
                    && !assignment.operator.is_logical()
                    && self.unused_expression(parent)
            }
            AstKind::UpdateExpression(_) => self.unused_expression(parent),
            _ => false,
        };
        self_update
            || rhs.is_some_and(|rhs| {
                contains(rhs, reference.span) && !self.inside_storable_function(reference.node, rhs)
            })
    }

    fn rhs_node(&self, reference: &Ref, previous: Option<Span>, home: ScopeId) -> Option<Span> {
        if let Some(previous) = previous
            && contains(previous, reference.span)
        {
            return Some(previous);
        }
        let parent = self.parent(reference.node);
        let AstKind::AssignmentExpression(assignment) = self.nodes.kind(parent) else {
            return None;
        };
        let later = self.variable_scope(reference.scope) != home || self.in_loop(reference.node);
        (assignment.left.span() == reference.span && self.unused_expression(parent) && !later)
            .then(|| assignment.right.span())
    }

    fn in_loop(&self, node: NodeId) -> bool {
        for ancestor in self.nodes.ancestor_ids(node) {
            let kind = self.nodes.kind(ancestor);
            if kind.is_function_like() {
                return false;
            }
            if kind.is_iteration_statement() {
                return true;
            }
        }
        false
    }

    fn inside_storable_function(&self, node: NodeId, rhs: Span) -> bool {
        let Some(function) = self
            .nodes
            .ancestor_ids(node)
            .find(|ancestor| self.nodes.kind(*ancestor).is_function_like())
        else {
            return false;
        };
        contains(rhs, self.span(function)) && self.storable(function, rhs)
    }

    /// ESLint `isStorableFunction`: the function value may outlive the write.
    fn storable(&self, function: NodeId, rhs: Span) -> bool {
        let mut node_span = self.span(function);
        let mut current = function;
        loop {
            let parent = self.nodes.parent_id(current);
            if parent == current || !contains(rhs, self.span(parent)) {
                return false;
            }
            let kind = self.nodes.kind(parent);
            match kind {
                AstKind::ParenthesizedExpression(_) => {
                    current = parent;
                    continue;
                }
                AstKind::SequenceExpression(sequence) => {
                    let last = sequence
                        .expressions
                        .last()
                        .map(|expression| expression.without_parentheses().span());
                    if last != Some(node_span) {
                        return false;
                    }
                }
                AstKind::CallExpression(call) => {
                    return call.callee.without_parentheses().span() != node_span;
                }
                AstKind::NewExpression(call) => {
                    return call.callee.without_parentheses().span() != node_span;
                }
                AstKind::AssignmentExpression(_)
                | AstKind::TaggedTemplateExpression(_)
                | AstKind::YieldExpression(_) => return true,
                _ if kind.is_statement() || kind.is_declaration() => return true,
                _ => {}
            }
            node_span = self.span(parent);
            current = parent;
        }
    }

    /// ESLint `canBecomeVariableDeclaration`, plus its destructuring checks.
    fn becomes_declaration(&self, node: NodeId, scope: ScopeId) -> bool {
        let mut current = node;
        let mut destructured = false;
        loop {
            let parent = self.nodes.parent_id(current);
            if parent == current {
                return false;
            }
            match self.nodes.kind(parent) {
                AstKind::ArrayAssignmentTarget(_)
                | AstKind::ObjectAssignmentTarget(_)
                | AstKind::AssignmentTargetRest(_)
                | AstKind::AssignmentTargetWithDefault(_)
                | AstKind::AssignmentTargetPropertyIdentifier(_)
                | AstKind::AssignmentTargetPropertyProperty(_) => {
                    destructured = true;
                    current = parent;
                }
                AstKind::AssignmentExpression(assignment) => {
                    if assignment.operator != AssignmentOperator::Assign
                        || (destructured && !self.local_targets(&assignment.left, scope))
                    {
                        return false;
                    }
                    let statement = self.parent(parent);
                    if !matches!(self.nodes.kind(statement), AstKind::ExpressionStatement(_)) {
                        return false;
                    }
                    return matches!(
                        self.nodes.parent_kind(statement),
                        AstKind::Program(_)
                            | AstKind::BlockStatement(_)
                            | AstKind::FunctionBody(_)
                            | AstKind::StaticBlock(_)
                            | AstKind::SwitchCase(_)
                    );
                }
                _ => return false,
            }
        }
    }

    /// Every destructuring target is a plain name declared in `scope`.
    fn local_targets(&self, target: &AssignmentTarget<'_>, scope: ScopeId) -> bool {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(id) => {
                let reference = self.scoping.get_reference(id.reference_id());
                reference
                    .symbol_id()
                    .is_none_or(|symbol| self.scoping.symbol_scope_id(symbol) == scope)
            }
            AssignmentTarget::ArrayAssignmentTarget(array) => self.local_array(array, scope),
            AssignmentTarget::ObjectAssignmentTarget(object) => self.local_object(object, scope),
            _ => false,
        }
    }

    fn local_array(&self, array: &ArrayAssignmentTarget<'_>, scope: ScopeId) -> bool {
        array
            .elements
            .iter()
            .flatten()
            .all(|element| self.local_maybe_default(element, scope))
            && array
                .rest
                .as_ref()
                .is_none_or(|rest| self.local_targets(&rest.target, scope))
    }

    fn local_object(&self, object: &ObjectAssignmentTarget<'_>, scope: ScopeId) -> bool {
        object.properties.iter().all(|property| match property {
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(id) => {
                let reference = self.scoping.get_reference(id.binding.reference_id());
                reference
                    .symbol_id()
                    .is_none_or(|symbol| self.scoping.symbol_scope_id(symbol) == scope)
            }
            AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                self.local_maybe_default(&property.binding, scope)
            }
        }) && object
            .rest
            .as_ref()
            .is_none_or(|rest| self.local_targets(&rest.target, scope))
    }

    fn local_maybe_default(
        &self,
        target: &AssignmentTargetMaybeDefault<'_>,
        scope: ScopeId,
    ) -> bool {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                self.local_targets(&with_default.binding, scope)
            }
            other => other
                .as_assignment_target()
                .is_some_and(|target| self.local_targets(target, scope)),
        }
    }
}

fn contains(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}
