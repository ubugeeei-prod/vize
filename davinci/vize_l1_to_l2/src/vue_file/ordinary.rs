//! Only the original default statement's staged Options refusal may resolve.

use super::{VueFileIssue, VueFileIssueKind, VueScriptReceipt};
use vize_l2::lang::js::OrdinaryEmptyScript;

pub(super) fn staged_options(
    ordinary: Option<VueScriptReceipt>,
    setup: Option<VueScriptReceipt>,
    family: Option<OrdinaryEmptyScript<'_>>,
) -> Option<VueFileIssue> {
    if setup.is_some() {
        return None;
    }
    let ordinary = ordinary?;
    let family = family?;
    if ordinary.unit() != family.unit()
        || ordinary.scope() != family.scope()
        || ordinary.span() != family.script_span()
    {
        return None;
    }
    Some(VueFileIssue {
        unit: Some(family.unit()),
        scope: Some(family.scope()),
        span: family.statement_span(),
        kind: VueFileIssueKind::UnsupportedOptions,
    })
}
