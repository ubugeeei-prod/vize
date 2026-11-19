//! The actual native For seal already checked selected setup membership.
//!
//! This classifies only its retained collection row. It never repeats whole
//! Program eligibility, resolves names or manufactures an expression context.

use crate::decision::dom::{
    DomFileForHead,
    vue::{VueReadKind, VueRenderRead},
};
use vize_l2::{
    file::{DeclarationKind, Namespace},
    resolution::Usage,
};

pub(super) fn classify<'owner, 'arena>(
    row: &DomFileForHead<'owner, 'arena>,
) -> Option<VueRenderRead<'owner, 'arena>> {
    let occurrence = row.resolution().collection_occurrence();
    let binding = row.collection();
    // Original parent aliases are genuine declarations, but they have no setup
    // ScriptUnit or runtime naming authority and must remain a separate family.
    let declaration = binding.declaration()?;
    if occurrence.usage != Usage::Read
        || occurrence.binding != binding.id()
        || declaration.id != binding.id()
        || declaration.name != occurrence.name
        || declaration.namespace != Namespace::Value
        || !declaration.is_direct_program()
    {
        return None;
    }
    let kind = match declaration.kind {
        DeclarationKind::Const if declaration.initializer.is_primitive() => VueReadKind::SetupConst,
        DeclarationKind::Let | DeclarationKind::Var => VueReadKind::SetupLet,
        _ => return None,
    };
    Some(VueRenderRead {
        occurrence,
        binding,
        kind,
    })
}
