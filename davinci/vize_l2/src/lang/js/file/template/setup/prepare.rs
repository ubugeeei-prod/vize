//! Check whole original setup eligibility before entering the original cursor.

use super::{
    TemplateRouteState, SelectedSetupIssue, SelectedSetupIssueKind, TemplateRouteIssue,
    TemplateOwner,
};
use crate::lang::js::{TemplateRouteIssueKind, FileTemplateWalk};

impl<'a> TemplateOwner<'a> {
    /// Begin the existing sole native root cursor only after this original
    /// normally owned setup and its completed declaration unit match. This is
    /// not a whole descriptor envelope or output capability. The ordinary
    /// diagnostic `begin()` retains its existing separate contract.
    pub fn begin_setup(&mut self) -> Result<FileTemplateWalk<'_, 'a>, TemplateRouteIssue> {
        let span = self.selected.component().block().span();
        if !matches!(self.state, TemplateRouteState::Scripts) {
            return self.refuse(span, TemplateRouteIssueKind::Interrupted);
        }
        self.state = TemplateRouteState::Interrupted;
        if let Err(issue) = ready(self) {
            return self.refuse(issue.span, TemplateRouteIssueKind::SetupPolicy(issue.kind));
        }
        self.state = TemplateRouteState::Scripts;
        self.begin()
    }
}
fn ready(owner: &TemplateOwner<'_>) -> Result<(), SelectedSetupIssue> {
    let span = owner
        .selected()
        .setup()
        .map_or(owner.selected().component().block().span(), |script| {
            script.block().span()
        });
    let reject = |kind| SelectedSetupIssue { span, kind };
    let syntax = owner
        .retained_setup()
        .ok_or_else(|| reject(SelectedSetupIssueKind::MissingOwner))?;
    let joined = super::identity::checked(
        owner.selected(),
        syntax,
        &owner.producer.builder.facts.units,
        &owner.producer.builder.facts.scopes,
        owner.producer.builder.source,
    )?;
    if owner.setup != Some(joined.unit.id) {
        return Err(reject(SelectedSetupIssueKind::MissingUnit));
    }
    Ok(())
}
