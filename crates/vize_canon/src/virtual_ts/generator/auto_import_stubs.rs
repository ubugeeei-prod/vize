use vize_carton::{FxHashSet, String};
use vize_croquis::Croquis;

use super::super::types::VirtualTsOptions;
use super::imports::extract_declared_name;

/// Emit auto-import stubs (e.g. Nuxt composables).
///
/// Only names that are not already declared via imports or script bindings get
/// a stub. `imported_names` carries every module-level import so plain
/// `<script>` imports are covered too: `summary.bindings` only holds
/// `<script setup>` bindings when both blocks exist.
pub(super) fn emit_auto_import_stubs(
    ts: &mut String,
    summary: &Croquis,
    options: &VirtualTsOptions,
    imported_names: &FxHashSet<&str>,
) {
    let mut has_header = false;
    for stub in &options.auto_import_stubs {
        let name = extract_declared_name(stub);
        if let Some(name) = name {
            // Skip if already imported or declared in script bindings
            if summary.bindings.bindings.contains_key(name) || imported_names.contains(&name) {
                continue;
            }
        }
        if !has_header {
            ts.push_str("\n// Auto-import stubs (framework-provided globals)\n");
            has_header = true;
        }
        ts.push_str(stub);
        ts.push('\n');
    }
}
