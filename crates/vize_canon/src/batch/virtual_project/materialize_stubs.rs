//! Writing the program-wide ambient stub files of the virtual project:
//! framework-provided globals, external module augmentations, the `.vue` module
//! marker, and the shared helper preamble.
//!
//! These are the parts of the materialized tree that no source file maps to, so
//! they are kept apart from [`super::materialize`], which owns the mirrored
//! source tree itself.

use serde_json::Value;
use vize_carton::String as CompactString;

use crate::batch::error::CorsaResult;
use crate::batch::materialize_fs::write_if_changed;

use super::{
    AUTO_IMPORT_STUBS_FILE, MODULE_AUGMENTATION_STUB_PREFIX, MODULE_AUGMENTATION_STUBS_FILE,
    SHARED_HELPERS_FILE, VUE_MODULE_STUBS_FILE, VirtualProject,
};

impl VirtualProject {
    pub(super) fn write_auto_import_stubs(&self) -> CorsaResult<()> {
        if !self.has_global_auto_import_stubs() {
            return Ok(());
        }

        let capacity = self
            .virtual_ts_options
            .auto_import_stubs
            .iter()
            .filter(|stub| !is_module_augmentation_stub(stub))
            .fold(64usize, |acc, stub| acc + stub.len() + 1);
        let mut content = CompactString::with_capacity(capacity);
        content.push_str("// @ts-nocheck\n");
        content.push_str("// Framework-provided globals for the virtual project.\n");
        for stub in &self.virtual_ts_options.auto_import_stubs {
            if is_module_augmentation_stub(stub) {
                continue;
            }
            content.push_str(stub);
            content.push('\n');
        }

        write_if_changed(
            &self.virtual_root.join(AUTO_IMPORT_STUBS_FILE),
            content.as_bytes(),
        )?;
        Ok(())
    }

    pub(super) fn write_module_augmentation_stubs(&self) -> CorsaResult<()> {
        if !self.has_module_augmentation_stubs() {
            return Ok(());
        }

        let capacity = self
            .virtual_ts_options
            .auto_import_stubs
            .iter()
            .filter(|stub| is_module_augmentation_stub(stub))
            .fold(96usize, |acc, stub| acc + stub.len() + 1);
        let mut content = CompactString::with_capacity(capacity);
        content.push_str("// @ts-nocheck\n");
        content.push_str("// External module augmentations for resolved framework packages.\n");
        content.push_str("export {};\n");
        for stub in &self.virtual_ts_options.auto_import_stubs {
            if !is_module_augmentation_stub(stub) {
                continue;
            }
            content.push_str(stub.trim_start_matches(MODULE_AUGMENTATION_STUB_PREFIX));
            content.push('\n');
        }

        write_if_changed(
            &self.virtual_root.join(MODULE_AUGMENTATION_STUBS_FILE),
            content.as_bytes(),
        )?;
        Ok(())
    }

    pub(super) fn write_vue_module_stubs(&self) -> CorsaResult<()> {
        let content = "// Vue SFC modules resolve through materialized .vue.ts files.\n";
        write_if_changed(
            &self.virtual_root.join(VUE_MODULE_STUBS_FILE),
            content.as_bytes(),
        )?;
        Ok(())
    }

    /// Write the shared ambient helpers file. The generated `.vue.ts` modules
    /// hoist their common preamble (ImportMeta augmentation, type helpers,
    /// compiler-macro signatures) into this single program-wide declaration.
    pub(super) fn write_shared_helpers(&self) -> CorsaResult<()> {
        let mut content = CompactString::default();
        if self.needs_vue_jsx_reference() {
            content.push_str("/// <reference types=\"vue/jsx\" />\n");
        }
        content.push_str(crate::virtual_ts::SHARED_PREAMBLE_DTS);
        write_if_changed(
            &self.virtual_root.join(SHARED_HELPERS_FILE),
            content.as_bytes(),
        )?;
        Ok(())
    }

    fn needs_vue_jsx_reference(&self) -> bool {
        if self.needs_vue_jsx_compiler_options() {
            return true;
        }
        let Ok(options) = self.load_compiler_options(self.resolved_tsconfig_path().as_deref())
        else {
            return false;
        };
        options
            .get("jsxImportSource")
            .and_then(Value::as_str)
            .is_some_and(|source| source == "vue")
            || options
                .get("types")
                .and_then(Value::as_array)
                .is_some_and(|types| {
                    types
                        .iter()
                        .any(|entry| entry.as_str().is_some_and(|entry| entry == "vue/jsx"))
                })
    }

    pub(super) fn has_global_auto_import_stubs(&self) -> bool {
        self.virtual_ts_options
            .auto_import_stubs
            .iter()
            .any(|stub| !is_module_augmentation_stub(stub))
    }

    pub(super) fn has_module_augmentation_stubs(&self) -> bool {
        self.virtual_ts_options
            .auto_import_stubs
            .iter()
            .any(|stub| is_module_augmentation_stub(stub))
    }

    pub(super) fn push_stub_include_paths(&self, includes: &mut Vec<CompactString>) {
        if self.has_global_auto_import_stubs() {
            includes.push(AUTO_IMPORT_STUBS_FILE.into());
        }
        if self.has_module_augmentation_stubs() {
            includes.push(MODULE_AUGMENTATION_STUBS_FILE.into());
        }
        includes.push(VUE_MODULE_STUBS_FILE.into());
    }
}

fn is_module_augmentation_stub(stub: &str) -> bool {
    stub.starts_with(MODULE_AUGMENTATION_STUB_PREFIX)
}
