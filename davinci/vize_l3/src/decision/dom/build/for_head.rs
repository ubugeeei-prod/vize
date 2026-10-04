//! Direct same-owner joins only; no second AST, header or canonical walk.

use super::{DomBuilder, DomExpressionFacts};
use crate::decision::dom::{DomUnsupported, for_head::DomFileForHead, vue::policy::FileReads};
use vize_l0::id::NodeId;
use vize_l2::{
    file::{BindingRef, FileArtifact, ScopeId},
    op::OriginalForOp,
    resolution::ForAliasDeclaration,
};
mod read;
mod runtime;

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
    pub(super) fn record_for_head(
        &mut self,
        node: NodeId,
        original: &'owner OriginalForOp<'arena>,
    ) -> bool {
        let Some(file) = self.file else { return false };
        match join(file, node, original) {
            Ok(mut row) => {
                row.read = read::classify(&row);
                if row.read.is_none() {
                    self.reject(
                        node,
                        row.resolution().collection_authored_span(),
                        DomUnsupported::ForCollectionAccess,
                    );
                }
                let runtime = self.prepare_for_runtime(node, &row);
                self.facts.file_for_heads.insert(node, row);
                runtime
            }
            Err(reason) => {
                self.reject(node, original.span, reason);
                false
            }
        }
    }
}

fn join<'owner, 'arena>(
    file: &'owner FileArtifact<'arena>,
    node: NodeId,
    original: &'owner OriginalForOp<'arena>,
) -> Result<DomFileForHead<'owner, 'arena>, DomUnsupported> {
    let head = file
        .for_head_for(original)
        .filter(|head| head.id().node() == node)
        .ok_or(DomUnsupported::FileForHead)?;
    let resolution = head.resolution().ok_or(DomUnsupported::FileForHead)?;
    if !core::ptr::eq(
        resolution
            .input()
            .operand()
            .syntax()
            .source()
            .authored_root(),
        file.artifact().source(),
    ) {
        return Err(DomUnsupported::FileForHead);
    }
    let enclosing = head.enclosing_scope().ok_or(DomUnsupported::FileScope)?;
    let scope = head.scope().ok_or(DomUnsupported::FileScope)?;
    let outer = file
        .scopes()
        .get(enclosing.index() as usize)
        .filter(|row| row.id == enclosing)
        .ok_or(DomUnsupported::FileScope)?;
    let child = file
        .scopes()
        .get(scope.index() as usize)
        .filter(|row| row.id == scope)
        .ok_or(DomUnsupported::FileScope)?;
    if child.parent != Some(outer.id) {
        return Err(DomUnsupported::FileScope);
    }
    if !alias(
        head.value(),
        &resolution.value_declaration(),
        file,
        head.id(),
        scope,
    ) || match (head.key(), resolution.key_declaration()) {
        (None, None) => false,
        (Some(binding), Some(declaration)) => {
            !alias(Some(binding), &declaration, file, head.id(), scope)
        }
        _ => true,
    } {
        return Err(DomUnsupported::FileBinding);
    }
    let occurrence = resolution.collection();
    let collection = file
        .binding(occurrence.binding)
        .ok_or(DomUnsupported::FileBinding)?;
    Ok(DomFileForHead {
        head,
        original,
        resolution,
        collection,
        read: None,
    })
}

fn alias(
    binding: Option<BindingRef<'_, '_>>,
    expected: &ForAliasDeclaration<'_, '_>,
    file: &FileArtifact<'_>,
    origin: vize_l2::op::OriginalForId,
    scope: ScopeId,
) -> bool {
    let Some(binding) = binding else { return false };
    let Some(row) = binding.template_declaration() else {
        return false;
    };
    let declaration = row.declaration();
    let Some(original) = declaration.original() else {
        return false;
    };
    core::ptr::eq(binding.file(), file)
        && core::ptr::eq(row.file(), file)
        && declaration.id() == binding.id()
        && declaration.origin() == origin
        && declaration.scope() == scope
        && core::ptr::eq(original.resolution(), expected.resolution())
        && core::ptr::eq(original.parameter(), expected.parameter())
        && core::ptr::eq(original.pattern(), expected.pattern())
        && original.fact() == expected.fact()
}

#[cfg(test)]
mod tests;
