//! The existing unresolved-reference and template-scope owner, including call roles.
use super::Replacement;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use oxc_ast::ast::{CallExpression, Expression, IdentifierReference, ObjectProperty};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_object_property},
};
use oxc_semantic::Scoping;
use vize_carton::{FxHashSet, String};
use vize_croquis::ScopeKind;
use vize_croquis::{ScopeChain, ScopeId};

pub(super) struct ReferenceCollector<'a> {
    pub(super) expression_len: usize,
    pub(super) offset: u32,
    pub(super) replacements: Vec<Replacement>,
    pub(super) scoping: &'a Scoping,
    pub(super) template_binding_access: &'a TemplateBindingAccess<'a>,
    pub(super) template_scopes: &'a ScopeChain,
    pub(super) usage_scope_id: ScopeId,
    pub(super) only_instance: bool,
    pub(super) callees: FxHashSet<(u32, u32)>,
}

impl ReferenceCollector<'_> {
    fn replacement(&self, ident: &IdentifierReference<'_>, shorthand: bool) -> Option<Replacement> {
        let authored_callee = self.only_instance
            && self.callees.contains(&(ident.span.start, ident.span.end))
            && self
                .template_binding_access
                .is_authored_instance_callee(ident.name.as_str());
        if !authored_callee && !self.template_binding_access.contains(ident.name.as_str()) {
            return None;
        }
        let instance = self
            .template_binding_access
            .is_instance_global(ident.name.as_str());
        if self.only_instance && !instance && !authored_callee {
            return None;
        }
        if is_visible_template_binding(
            self.template_scopes,
            self.usage_scope_id,
            ident.name.as_str(),
        ) {
            return None;
        }
        let reference_id = ident.reference_id.get()?;
        let reference = self.scoping.get_reference(reference_id);
        if reference.symbol_id().is_some()
            || (reference.flags().is_type() && !reference.flags().is_value_as_type())
            || (reference.flags().is_value_as_type() && !instance)
        {
            return None;
        }

        let start = ident.span.start.checked_sub(self.offset)? as usize;
        let end = ident.span.end.checked_sub(self.offset)? as usize;
        (start < end && end <= self.expression_len).then_some(Replacement {
            source: start..end,
            name: String::from(ident.name.as_str()),
            receiver_shadowed: instance
                && (self
                    .scoping
                    .find_binding(reference.scope_id(), "__ctx".into())
                    .is_some_and(|symbol| self.scoping.symbol_flags(symbol).is_value())
                    || is_visible_template_binding(
                        self.template_scopes,
                        self.usage_scope_id,
                        "__ctx",
                    )),
            shorthand,
            authored_callee,
        })
    }
}

fn is_visible_template_binding(scopes: &ScopeChain, start: ScopeId, name: &str) -> bool {
    let mut current = Some(start);
    while let Some(id) = current {
        let Some(scope) = scopes.get_scope(id) else {
            return false;
        };
        if matches!(
            scope.kind,
            ScopeKind::VFor | ScopeKind::VSlot | ScopeKind::EventHandler
        ) && scope.has_binding(name)
        {
            return true;
        }
        current = scope.parent();
    }
    false
}

impl<'a> Visit<'a> for ReferenceCollector<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.template_binding_access.has_authored_instance_calls()
            && let Expression::Identifier(ident) = call.callee.get_inner_expression()
        {
            self.callees.insert((ident.span.start, ident.span.end));
        }
        walk_call_expression(self, call);
    }

    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if let Some(replacement) = self.replacement(ident, false) {
            self.replacements.push(replacement);
        }
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if property.shorthand
            && let oxc_ast::ast::Expression::Identifier(ident) = &property.value
            && let Some(replacement) = self.replacement(ident, true)
        {
            self.replacements.push(replacement);
            return;
        }
        walk_object_property(self, property);
    }
}
