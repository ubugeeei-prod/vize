//! Template-only public instance bindings.

use super::VirtualTsGenerator;

impl VirtualTsGenerator {
    /// Emit public instance attributes in the template's lexical scope.
    pub(crate) fn emit_template_globals(&mut self) {
        self.emit_line("const $attrs: Readonly<Record<string, unknown>> = {};");
    }
}
