//! Setup values: `<script setup>` locals that module-scope types cannot name.

use super::{CompactString, FxHashMap, FxHashSet, ImportStatementInfo, ScriptParseResult};
use crate::script_parser::extract::value_prop_type_roots;
use vize_relief::BindingType;

/// A binding is a setup value unless an import or a normal `<script>` export
/// declares it at module scope.
#[inline]
pub(super) fn is_setup_value(
    bindings: &FxHashMap<CompactString, BindingType>,
    module_values: &FxHashSet<CompactString>,
    spans: &FxHashMap<CompactString, (u32, u32)>,
    imports: &[ImportStatementInfo],
    key: &str,
) -> bool {
    bindings.contains_key(key)
        && !module_values.contains(key)
        && !spans.get(key).is_some_and(|(start, end)| {
            // Bindings whose declaration site falls inside an import
            // statement are module-scoped imports, not setup values.
            imports
                .iter()
                .any(|imp| *start >= imp.start && *end <= imp.end)
        })
}

impl ScriptParseResult {
    /// The props type lives at module scope, so a runtime prop typed from a
    /// setup value (which Vue rejects in `defineProps`) stays untyped.
    #[inline]
    pub(crate) fn forget_setup_value_prop_types(&mut self) {
        if !self.reads_prop_values || self.is_non_setup_script {
            return;
        }
        let (bindings, module_values, spans, imports) = (
            &self.bindings.bindings,
            &self.module_value_bindings,
            &self.binding_spans,
            &self.import_statements,
        );
        self.macros.forget_prop_types(|prop_type| {
            value_prop_type_roots(prop_type)
                .any(|root| is_setup_value(bindings, module_values, spans, imports, root))
        });
    }
}
