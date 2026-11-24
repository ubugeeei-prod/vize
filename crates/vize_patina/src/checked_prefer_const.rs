//! Default `vue/prefer-const` for an original, closed script-only component.
use crate::LintDiagnostic;
use oxc_ast::ast::VariableDeclarationKind;
use vize_l0::{FxHashSet, cstr};
use vize_l1::container::{ContainerSegment, vue::L1Ir};
use vize_l2::query::script::assignments::CheckedAssignments;

/// Identifier used by the stock Patina rule and its diagnostics.
pub const RULE: &str = "vue/prefer-const";

/// Reason a complete component cannot attest to all writes for this rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferConstRefusal {
    /// The original Vue block structure was rejected.
    Descriptor,
    /// Additional blocks or interstitial content can carry writes or suppressions.
    Envelope,
    /// The supplied proof belongs to a different physical block or grammar.
    ForeignScript,
    /// One or more statements can introduce a write the checker did not examine.
    OpenScript,
    /// The script may carry a lint directive.
    Comments,
    /// A repeated or escaped spelling requires semantic name resolution.
    BindingIdentity,
}

/// All admitted executable statements initialize distinct simple identifiers
/// from literals. There are no expressions that can write a binding later.
/// Requiring one actual script and whitespace around it also excludes sibling
/// reads/writes, template directives and file-level lint suppressions.
pub fn default_prefer_const(
    owner: &L1Ir<'_>,
    checked: &CheckedAssignments<'_, '_, '_>,
    mut report: impl FnMut(LintDiagnostic),
) -> Result<(), PreferConstRefusal> {
    let view = owner
        .admitted()
        .map_err(|_| PreferConstRefusal::Descriptor)?;
    let script = match (view.ordinary(), view.setup()) {
        (Some(script), None) | (None, Some(script)) => script,
        _ => return Err(PreferConstRefusal::Envelope),
    };
    if owner.container().blocks.len() != 1 || owner.container().segments().any(|segment| {
        matches!(segment, ContainerSegment::Interstitial(span) if !span.slice(owner.source()).trim().is_empty())
    }) {
        return Err(PreferConstRefusal::Envelope);
    }
    let block = checked.source();
    if script.lang() != vize_l1::embed::Lang::Ts
        || checked.original().source_type() != oxc_span::SourceType::ts()
        || !core::ptr::eq(block.root_source(), view.source())
        || !core::ptr::eq(block.source(), script.block().source())
        || block.span() != script.block().span()
    {
        return Err(PreferConstRefusal::ForeignScript);
    }
    if !checked.is_complete() {
        return Err(PreferConstRefusal::OpenScript);
    }
    if !checked.original().parsed().program.comments.is_empty() {
        return Err(PreferConstRefusal::Comments);
    }
    let mut names = FxHashSet::default();
    for row in checked.assignments() {
        let name = row.binding.slice(view.source());
        if name.contains('\\') || !names.insert(name) {
            return Err(PreferConstRefusal::BindingIdentity);
        }
    }
    for row in checked.assignments().filter(|row| {
        !row.exported
            && matches!(
                row.kind,
                VariableDeclarationKind::Let | VariableDeclarationKind::Var
            )
    }) {
        let name = row.binding.slice(view.source());
        report(LintDiagnostic::warn(
            RULE,
            cstr!("'{name}' is never reassigned. Use 'const' instead"),
            row.binding.start,
            row.binding.end,
        ));
    }
    Ok(())
}
