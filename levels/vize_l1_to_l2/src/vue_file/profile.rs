//! Intrinsic ordinary template profile from original script observations only.

use super::{VueFileIssue, VueFileIssueKind, VueScriptReceipt};
use oxc_span::SourceType;
use vize_l1::embed::Lang;

#[derive(Clone, Copy)]
pub(crate) struct NativeTemplateProfile(SourceType);

impl NativeTemplateProfile {
    pub(super) fn checked(
        ordinary: Option<VueScriptReceipt>,
        setup: Option<VueScriptReceipt>,
    ) -> Result<Self, VueFileIssue> {
        for receipt in [ordinary, setup].into_iter().flatten() {
            let profile = receipt.source_type;
            if !profile.is_module() || profile.is_jsx() || profile.is_typescript_definition() {
                return Err(issue(receipt, VueFileIssueKind::InvalidTemplateProfile));
            }
        }
        if let Some((ordinary, setup)) = ordinary.zip(setup)
            && ordinary.source_type.is_typescript() != setup.source_type.is_typescript()
        {
            return Err(issue(setup, VueFileIssueKind::ConflictingScriptProfiles));
        }
        Ok(Self(
            setup
                .or(ordinary)
                .map_or(SourceType::mjs(), |receipt| receipt.source_type),
        ))
    }
    pub(crate) fn lang(self) -> Lang {
        if self.0.is_typescript() {
            Lang::Ts
        } else {
            Lang::Js
        }
    }
}

fn issue(receipt: VueScriptReceipt, kind: VueFileIssueKind) -> VueFileIssue {
    VueFileIssue {
        unit: Some(receipt.unit()),
        scope: Some(receipt.scope()),
        span: receipt.span(),
        kind,
    }
}
