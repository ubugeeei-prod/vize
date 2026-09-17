use vize_carton::{String, config::VueVersion};
use vize_croquis::{Croquis, ScopeKind};

/// Standalone native documents cannot rely on the program-wide ambient file.
pub(super) fn emit_embedded_helpers(
    ts: &mut String,
    summary: &Croquis,
    legacy_vue2: bool,
    dialect: VueVersion,
) {
    ts.push_str(super::legacy_vue2::vue_type_helpers(legacy_vue2, dialect));
    if summary
        .scopes
        .iter()
        .any(|scope| scope.kind == ScopeKind::VMatch)
    {
        ts.push_str(include_str!("../helpers/pattern_matching.d.ts"));
        ts.push('\n');
    }
}
