//! Preserve the established semantic Debug projection.
//!
//! Source-owned authority carriers are private analysis state. They must not
//! appear in whole-output receipts or make demand-only captures change output.

use super::Croquis;
use core::fmt;

impl fmt::Debug for Croquis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Croquis")
            .field("scopes", &self.scopes)
            .field("symbols", &self.symbols)
            .field("macros", &self.macros)
            .field("reactivity", &self.reactivity)
            .field("race_conditions", &self.race_conditions)
            .field("provide_inject", &self.provide_inject)
            .field("setup_context", &self.setup_context)
            .field("types", &self.types)
            .field("hoists", &self.hoists)
            .field("bindings", &self.bindings)
            .field("component_shape", &self.component_shape)
            .field("template_info", &self.template_info)
            .field("used_components", &self.used_components)
            .field("component_registrations", &self.component_registrations)
            .field(
                "template_component_registrations",
                &self.template_component_registrations,
            )
            .field("component_usages", &self.component_usages)
            .field("used_directives", &self.used_directives)
            .field("undefined_refs", &self.undefined_refs)
            .field("unused_bindings", &self.unused_bindings)
            .field("type_exports", &self.type_exports)
            .field("invalid_exports", &self.invalid_exports)
            .field("template_expressions", &self.template_expressions)
            .field("pattern_diagnostics", &self.pattern_diagnostics)
            .field("element_ids", &self.element_ids)
            .field("import_statements", &self.import_statements)
            .field("re_exports", &self.re_exports)
            .field("re_export_forwards", &self.re_export_forwards)
            .field("binding_spans", &self.binding_spans)
            .field("options_descriptor", &self.options_descriptor)
            .finish()
    }
}
