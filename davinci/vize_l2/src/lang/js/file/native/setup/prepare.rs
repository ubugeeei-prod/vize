//! Check whole original setup eligibility before entering the original cursor.

use super::{
    NativeRouteState, NativeSetupIssue, NativeSetupIssueKind, NativeTemplateIssue,
    NativeTemplateIssueKind, NativeTemplateOwner,
};
use crate::lang::js::NativeTemplateWalk;

impl<'a> NativeTemplateOwner<'a> {
    /// Begin the existing sole native root cursor only after this original
    /// normally owned setup and its completed declaration unit match. This is
    /// not a whole descriptor envelope or output capability. The ordinary
    /// diagnostic `begin()` retains its existing separate contract.
    pub fn begin_setup(&mut self) -> Result<NativeTemplateWalk<'_, 'a>, NativeTemplateIssue> {
        let span = self.selected.component().block().span();
        if !matches!(self.state, NativeRouteState::Scripts) {
            return self.refuse(span, NativeTemplateIssueKind::Interrupted);
        }
        self.state = NativeRouteState::Interrupted;
        if let Err(issue) = ready(self) {
            return self.refuse(issue.span, NativeTemplateIssueKind::SetupPolicy(issue.kind));
        }
        self.state = NativeRouteState::Scripts;
        self.begin()
    }
}
fn ready(owner: &NativeTemplateOwner<'_>) -> Result<(), NativeSetupIssue> {
    let span = owner
        .selected()
        .setup()
        .map_or(owner.selected().component().block().span(), |script| {
            script.block().span()
        });
    let reject = |kind| NativeSetupIssue { span, kind };
    let syntax = owner
        .retained_setup()
        .ok_or_else(|| reject(NativeSetupIssueKind::MissingOwner))?;
    let joined = super::identity::checked(
        owner.selected(),
        syntax,
        &owner.producer.builder.facts.units,
        &owner.producer.builder.facts.scopes,
        owner.producer.builder.source,
    )?;
    if owner.setup != Some(joined.unit.id) {
        return Err(reject(NativeSetupIssueKind::MissingUnit));
    }
    Ok(())
}
