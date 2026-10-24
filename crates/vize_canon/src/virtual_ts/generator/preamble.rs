use vize_carton::{String, config::VueVersion};
use vize_croquis::{Croquis, ScopeKind};

use crate::virtual_ts::{
    import_meta::emit_import_meta_augmentation, scope::emit_slot_payload_helpers,
    types::VirtualTsGenerationOptions,
};

/// Keep helpers outside setup so public component aliases can refer to them.
pub(super) fn emit_module_preamble(
    ts: &mut String,
    summary: &Croquis,
    options: VirtualTsGenerationOptions<'_>,
    legacy_vue2: bool,
) {
    if options.hoist_shared_preamble {
        // Program helpers and global augmentation are shared once per project.
        ts.push_str("// Shared preamble hoisted to the program-wide __vize_helpers.d.ts\n");
    } else {
        emit_import_meta_augmentation(ts, !options.omit_vite_client_reference);
        ts.push('\n');
    }
    ts.push_str("// ========== Module Scope (imports) ==========\n");
    if !options.hoist_shared_preamble {
        emit_embedded_helpers(
            ts,
            summary,
            legacy_vue2,
            options.dialect,
            options.external_component_attrs_module,
        );
    }
    emit_slot_payload_helpers(ts, summary, !options.hoist_shared_preamble);
}

/// Standalone native documents cannot rely on the program-wide ambient file.
fn emit_embedded_helpers(
    ts: &mut String,
    summary: &Croquis,
    legacy_vue2: bool,
    dialect: VueVersion,
    external_component_attrs_module: Option<&str>,
) {
    if !super::legacy_vue2::needs_legacy_vue2_helpers(legacy_vue2, dialect)
        && external_component_attrs_module
            == Some(crate::virtual_ts::helpers::CONTENT_MAPPER_HELPERS_MODULE)
    {
        ts.push_str(crate::virtual_ts::helpers::CONTENT_MAPPER_TYPE_HELPERS);
    } else {
        ts.push_str(super::legacy_vue2::vue_type_helpers(legacy_vue2, dialect));
    }
    if summary
        .scopes
        .iter()
        .any(|scope| scope.kind == ScopeKind::VMatch)
    {
        ts.push_str(include_str!("../helpers/pattern_matching.d.ts"));
        ts.push('\n');
    }
}
