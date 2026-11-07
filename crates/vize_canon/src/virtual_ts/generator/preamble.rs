use vize_carton::{String, config::VueVersion};
use vize_croquis::{Croquis, ScopeKind};

use crate::virtual_ts::{
    import_meta::emit_import_meta_augmentation,
    scope::emit_slot_payload_helpers,
    types::{ComponentHelperModule, VirtualTsGenerationOptions},
};

const OWNER_SENSITIVE_COMPONENT_INSTANCE_HELPERS: &str = concat!(
    include_str!("../helpers/declarations/core-types.d.ts.txt"),
    include_str!("../helpers/declarations/vue3-types.d.ts.txt"),
    include_str!("../helpers/declarations/prop-keys.d.ts.txt"),
    include_str!("../component_prop_helpers.txt")
);
const OWNER_SENSITIVE_COMPONENT_INSTANCE_HELPER_EXPORTS: &str = concat!(
    "export type { __RuntimePropShape, __LooseRequired, __DefineProps, ",
    "__WithDefaultsArgs, __WithDefaultsResult, __Ref, __ShallowRef, ",
    "$UnownedComponentAttrs, $Prettify, $ComponentFallthroughProps, ",
    "$RelaxedDynamicUnionExtra };\n"
);

/// Keep helpers outside setup so public component aliases can refer to them.
pub(super) fn emit_module_preamble(
    ts: &mut String,
    summary: &Croquis,
    options: VirtualTsGenerationOptions<'_>,
    legacy_vue2: bool,
) {
    if options.hoist_shared_preamble {
        if matches!(
            options.external_component_attrs_module,
            Some(ComponentHelperModule::Types)
        ) {
            // Pulls the checker globals and helpers into the host program once.
            // Declaration emit drops this import, so consumers see only helpers.
            ts.push_str("import type {} from \"vize/virtual-ts-checker\";\n");
        } else {
            // Program helpers and global augmentation are shared once per project.
            ts.push_str("// Shared preamble hoisted to the program-wide __vize_helpers.d.ts\n");
        }
        if options.local_component_ownership_helpers {
            // TypeScript resolves property references through mapped-type
            // declarations. Sharing this alias bundle makes unrelated editor
            // SFCs appear to own the same prop symbol, so retain only this small
            // owner boundary in each generated editor module. Batch/declaration
            // projects keep their established fully-shared preamble.
            ts.push_str("// Keep public prop owners module-local for references and rename.\n");
            crate::virtual_ts::helpers::hygiene::emit_type_helpers(
                ts,
                summary,
                OWNER_SENSITIVE_COMPONENT_INSTANCE_HELPERS,
                false,
            );
            // Export helper aliases that are valid but optional for a given
            // SFC. TypeScript otherwise reports TS6196 under noUnusedLocals;
            // editor-only named type exports keep them module-local without
            // turning clean authored files into suggestion-only diagnostics.
            ts.push_str(OWNER_SENSITIVE_COMPONENT_INSTANCE_HELPER_EXPORTS);
        }
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
    external_component_attrs_module: Option<ComponentHelperModule>,
) {
    if !super::legacy_vue2::needs_legacy_vue2_helpers(legacy_vue2, dialect)
        && let Some(module) = external_component_attrs_module
    {
        let helpers = match module {
            ComponentHelperModule::Attributes => {
                crate::virtual_ts::helpers::LEGACY_CONTENT_MAPPER_TYPE_HELPERS
            }
            ComponentHelperModule::Types => crate::virtual_ts::helpers::CONTENT_MAPPER_TYPE_HELPERS,
        };
        crate::virtual_ts::helpers::hygiene::emit_type_helpers(
            ts,
            summary,
            helpers,
            matches!(module, ComponentHelperModule::Types),
        );
    } else {
        crate::virtual_ts::helpers::hygiene::emit_type_helpers(
            ts,
            summary,
            super::legacy_vue2::vue_type_helpers(legacy_vue2, dialect),
            false,
        );
    }
    if summary
        .scopes
        .iter()
        .any(|scope| scope.kind == ScopeKind::VMatch)
    {
        crate::virtual_ts::helpers::hygiene::emit_helper_text(
            ts,
            summary,
            include_str!("../helpers/pattern_matching.d.ts"),
        );
        ts.push('\n');
    }
}
