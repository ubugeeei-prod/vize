//! Actual template rows share the full File binding domain without script IDs.

use super::{Facts, FileIssueKind, Names, Scope, ScopeId, TemplateIssue};
use crate::file::for_head::{TemplateDeclaration, TemplateDeclarationRow};
use crate::op::OriginalForId;
use crate::resolution::{BindingId, ForAliasRole};
use vize_l0::{Span, String};

pub(super) fn script_binding_available(index: u32, template_count: usize) -> bool {
    if template_count == 0 {
        return true;
    }
    template_count
        .checked_sub(1)
        .and_then(|last| u32::try_from(last).ok())
        .and_then(|last| u32::MAX.checked_sub(last))
        .is_some_and(|lowest| index < lowest)
}

impl<'a> Facts<'a> {
    pub(crate) fn template_scope(
        &mut self,
        owner: OriginalForId,
        parent: ScopeId,
        span: Span,
    ) -> Result<ScopeId, FileIssueKind> {
        let reject = |facts: &mut Self, kind| {
            facts.template_issues.push(TemplateIssue {
                node: Some(owner.node()),
                span,
                kind,
            });
            kind
        };
        if !self
            .scopes
            .get(parent.index() as usize)
            .is_some_and(|scope| scope.id == parent)
        {
            return Err(reject(self, FileIssueKind::InvalidSpan));
        }
        let index = u32::try_from(self.scopes.len())
            .map_err(|_| reject(self, FileIssueKind::BindingLimit))?;
        let id = ScopeId(index);
        self.scopes.push(Scope {
            id,
            parent: Some(parent),
            span,
        });
        self.names.push(Names::default());
        Ok(id)
    }

    pub(crate) fn declare_template_alias(
        &mut self,
        owner: OriginalForId,
        role: ForAliasRole,
    ) -> Result<BindingId, FileIssueKind> {
        let record = self
            .for_heads
            .get(owner.node())
            .ok_or(FileIssueKind::InvalidSpan)?;
        let (original, previous) = match role {
            ForAliasRole::Value => (Some(record.resolution.value_declaration()), record.value),
            ForAliasRole::Key => (record.resolution.key_declaration(), record.key),
        };
        if previous.is_some() {
            return Err(FileIssueKind::DuplicateDeclaration);
        }
        let original = original.ok_or(FileIssueKind::InvalidSpan)?;
        let name = original.fact().name();
        let scope = record.scope;
        let index = u32::try_from(self.template_declarations.len())
            .map_err(|_| FileIssueKind::BindingLimit)?;
        let raw = u32::MAX - index;
        if self.declarations.len() > raw as usize {
            return Err(FileIssueKind::BindingLimit);
        }
        let names = self
            .names
            .get_mut(scope.index() as usize)
            .ok_or(FileIssueKind::InvalidSpan)?;
        if names.value_names.contains_key(name) {
            return Err(FileIssueKind::DuplicateDeclaration);
        }
        let id = BindingId::new(raw);
        names.value_names.insert(String::from(name), id);
        self.template_declarations
            .push(TemplateDeclarationRow { owner, role });
        let record = self
            .for_heads
            .get_mut(owner.node())
            .ok_or(FileIssueKind::InvalidSpan)?;
        match role {
            ForAliasRole::Value => record.value = Some(id),
            ForAliasRole::Key => record.key = Some(id),
        }
        Ok(id)
    }

    pub(crate) fn template_declaration(
        &self,
        id: BindingId,
    ) -> Option<TemplateDeclaration<'_, 'a>> {
        let row = self
            .template_declarations
            .get(u32::MAX.checked_sub(id.index())? as usize)?;
        let record = self.for_heads.get(row.owner.node())?;
        let binding = match row.role {
            ForAliasRole::Value => record.value,
            ForAliasRole::Key => record.key,
        };
        (binding == Some(id)).then_some(TemplateDeclaration { record, row, id })
    }
}

#[cfg(test)]
mod tests {
    use super::script_binding_available;

    #[test]
    fn full_binding_domain_boundary_never_wraps_or_reserves_a_sentinel() {
        assert!(script_binding_available(0, 0));
        assert!(script_binding_available(u32::MAX, 0));
        assert!(script_binding_available(0, 1));
        assert!(script_binding_available(u32::MAX - 1, 1));
        assert!(!script_binding_available(u32::MAX, 1));
        if usize::BITS > 32 {
            let max = usize::try_from(u32::MAX).unwrap();
            assert!(script_binding_available(0, max));
            assert!(!script_binding_available(1, max));
            assert!(!script_binding_available(0, max + 1));
            assert!(!script_binding_available(u32::MAX, max + 1));
            assert!(!script_binding_available(0, max + 2));
        }
    }
}
