//! A separate, nonserialized result of the existing script parse demand.

use super::Drawer;
use crate::script_parser::ModuleExportCapture;

impl Drawer {
    /// Observe module exports during the same original OXC parse.
    #[doc(hidden)]
    pub fn with_empty_module_exports(mut self) -> Self {
        let mut capture = ModuleExportCapture::default();
        if self.script_drawn {
            capture.refuse();
        }
        self.module_export_capture = Some(capture);
        self
    }

    /// Return ordinary Croquis unchanged and a bounded empty-export proof.
    #[doc(hidden)]
    pub fn finish_with_empty_module_exports(self) -> (crate::Croquis, bool) {
        let empty = self
            .module_export_capture
            .as_ref()
            .is_some_and(ModuleExportCapture::known_empty);
        (self.finish(), empty)
    }

    /// Return ordinary Croquis unchanged and every authored named export, or
    /// `None` when the module's export set is not provable from its own source.
    #[doc(hidden)]
    pub fn finish_with_module_exports(
        self,
    ) -> (
        crate::Croquis,
        Option<Vec<crate::script_parser::AuthoredModuleExport>>,
    ) {
        let exports = self
            .module_export_capture
            .as_ref()
            .and_then(ModuleExportCapture::known_exports);
        (self.finish(), exports)
    }
}

#[cfg(test)]
mod tests;
