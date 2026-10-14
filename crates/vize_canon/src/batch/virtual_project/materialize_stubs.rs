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
use crate::batch::materialize_fs::{ensure_dir, write_if_changed};

use super::{
    AUTO_IMPORT_STUBS_FILE, MODULE_AUGMENTATION_STUB_PREFIX, MODULE_AUGMENTATION_STUBS_FILE,
    VUE_MODULE_STUBS_FILE, VirtualProject,
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
            self.push_mirrored_stub(&mut content, stub);
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
        let path = self.shared_helpers_path();
        if let Some(parent) = path.parent() {
            ensure_dir(parent)?;
        }
        write_if_changed(&path, content.as_bytes())?;
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

impl VirtualProject {
    /// A global component typed by an authored script module (Nuxt's
    /// `components.d.ts` can name a barrel such as `form/index.ts`) must load
    /// its mirror. The original keeps unrewritten `.vue` imports, so loading it
    /// reports TS2307 that vue-tsc never sees.
    fn push_mirrored_stub(&self, content: &mut CompactString, stub: &str) {
        let mut copied = 0;
        let mut cursor = 0;
        while let Some(found) = stub.get(cursor..).and_then(|rest| rest.find("import(")) {
            let open = cursor + found + "import(".len();
            cursor = open;
            let Some(quote) = stub.get(open..open + 1).filter(|q| *q == "'" || *q == "\"") else {
                continue;
            };
            let start = open + 1;
            let Some(length) = stub.get(start..).and_then(|rest| rest.find(quote)) else {
                break;
            };
            let end = start + length;
            cursor = end;
            let Some(mirror) = stub
                .get(start..end)
                .and_then(|specifier| self.stub_mirror(specifier))
            else {
                continue;
            };
            content.push_str(stub.get(copied..start).unwrap_or_default());
            content.push_str(&mirror);
            copied = end;
        }
        content.push_str(stub.get(copied..).unwrap_or_default());
    }

    /// `./relative` for an absolute project specifier whose script module is
    /// mirrored at the same relative path, keeping the authored extension form.
    fn stub_mirror(&self, specifier: &str) -> Option<CompactString> {
        let path = std::path::Path::new(specifier);
        if !path.is_absolute() || specifier.contains("/node_modules/") {
            return None;
        }
        let relative = path.strip_prefix(&self.project_root).ok()?;
        const EXTENSIONS: [&str; 6] = ["", ".ts", ".tsx", ".mts", "/index.ts", "/index.tsx"];
        let mirrored = EXTENSIONS.iter().any(|extension| {
            let mut candidate = path.as_os_str().to_os_string();
            candidate.push(extension);
            let candidate = std::path::PathBuf::from(candidate);
            let mut virtual_candidate = self.virtual_root.join(relative).into_os_string();
            virtual_candidate.push(extension);
            candidate.is_file()
                && self
                    .original_index
                    .get(&candidate)
                    .is_some_and(|virtual_path| virtual_path.as_os_str() == virtual_candidate)
        });
        mirrored.then(|| {
            let mut rendered = CompactString::from("./");
            rendered.push_str(&relative.to_string_lossy().replace('\\', "/"));
            rendered
        })
    }
}

fn is_module_augmentation_stub(stub: &str) -> bool {
    stub.starts_with(MODULE_AUGMENTATION_STUB_PREFIX)
}
