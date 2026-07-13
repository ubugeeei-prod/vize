use vize_relief::{ReliefSnapshotNode, SnapshotFor, SnapshotIf, SnapshotIfBranch};
use vize_rendu::{RenduIfBranch, RenduNode, RenduNodeId};

use super::{
    super::{
        TemplateGraphAdapterError,
        provenance::rendu_provenance,
        rendu_helpers::{binding, optional_binding},
        scope::pattern_bindings,
    },
    RenduLowerer,
};

impl RenduLowerer<'_> {
    pub(super) fn lower_if(
        &mut self,
        node: &SnapshotIf,
    ) -> Result<RenduNodeId, TemplateGraphAdapterError> {
        let mut branches = Vec::with_capacity(node.branches().len());
        for id in node.branches() {
            let Some(ReliefSnapshotNode::IfBranch(branch)) = self.snapshot.node(*id) else {
                return Err(TemplateGraphAdapterError::ExpectedIfBranch(*id));
            };
            branches.push(self.lower_branch(branch)?);
        }
        Ok(self.builder.add_node(RenduNode::If {
            branches,
            provenance: rendu_provenance(&node.location, self.source),
        }))
    }

    pub(super) fn lower_standalone_branch(
        &mut self,
        branch: &SnapshotIfBranch,
    ) -> Result<RenduNodeId, TemplateGraphAdapterError> {
        let provenance = rendu_provenance(&branch.location, self.source);
        let branch = self.lower_branch(branch)?;
        Ok(self.builder.add_node(RenduNode::If {
            branches: vec![branch],
            provenance,
        }))
    }

    fn lower_branch(
        &mut self,
        branch: &SnapshotIfBranch,
    ) -> Result<RenduIfBranch, TemplateGraphAdapterError> {
        let condition = branch
            .condition
            .as_ref()
            .map(|condition| self.add_expression(condition));
        Ok(
            RenduIfBranch::new(condition, self.lower_nodes(branch.children())?)
                .with_provenance(rendu_provenance(&branch.location, self.source)),
        )
    }

    pub(super) fn lower_for(
        &mut self,
        node: &SnapshotFor,
    ) -> Result<RenduNodeId, TemplateGraphAdapterError> {
        let source = self.add_expression(&node.source);
        let provenance = rendu_provenance(&node.location, self.source);
        let value = binding(
            node.value_alias
                .as_ref()
                .or(node.parse_result.value.as_ref()),
            "_value",
            &provenance,
        );
        let key = optional_binding(
            node.key_alias.as_ref().or(node.parse_result.key.as_ref()),
            &provenance,
        );
        let index = optional_binding(
            node.object_index_alias
                .as_ref()
                .or(node.parse_result.index.as_ref()),
            &provenance,
        );
        let mut scope = pattern_bindings(&value.pattern);
        for binding in [key.as_ref(), index.as_ref()].into_iter().flatten() {
            scope.extend(pattern_bindings(&binding.pattern));
        }
        self.scopes.push(scope);
        let body = self.lower_nodes(node.children())?;
        self.scopes.pop();
        Ok(self.builder.add_node(RenduNode::For {
            source,
            value,
            key,
            index,
            key_expression: None,
            body,
            provenance,
        }))
    }
}
