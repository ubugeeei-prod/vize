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
            self.push_mirrored_stub(
                &mut content,
                stub.trim_start_matches(MODULE_AUGMENTATION_STUB_PREFIX),
            );
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
        content.push_str(self.shared_helpers_preamble());
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
    /// A stub that names an authored script module must load its mirror.
    ///
    /// A global component typed by a barrel (Nuxt's `components.d.ts` can name
    /// `form/index.ts`) would otherwise load the original, which keeps
    /// unrewritten `.vue` imports and reports TS2307 that vue-tsc never sees.
    /// An auto-imported type re-exported `from` its module would otherwise be a
    /// different module instance than the one a file imports directly.
    fn push_mirrored_stub(&self, content: &mut CompactString, stub: &str) {
        let mut copied = 0;
        let mut cursor = 0;
        while let Some((start, end)) = next_module_specifier(stub, cursor) {
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

    /// Rewrite generated reference directives through the original-source
    /// index, including explicitly registered installed declaration snapshots.
    fn mirrored_reference_directive(
        &self,
        line: &str,
        snapshots: &vize_carton::FxHashSet<&std::path::Path>,
    ) -> Option<CompactString> {
        let trimmed = line.trim_start();
        let value = trimmed.strip_prefix("/// <reference path=")?;
        let quote = value.chars().next()?;
        if !matches!(quote, '\'' | '"') {
            return None;
        }
        let path = value.get(1..)?.split_once(quote)?.0;
        let path = std::path::Path::new(path);
        if !snapshots.contains(path) {
            return None;
        }
        let file = self.find_by_original(path)?;
        let rendered = file.virtual_path.to_str()?;
        if rendered.contains(quote) || rendered.contains(['\r', '\n', '\0']) {
            return None;
        }
        Some(vize_carton::cstr!(
            "/// <reference path={quote}{rendered}{quote} />"
        ))
    }

    pub(in crate::batch) fn rewrite_snapshot_references(
        &mut self,
        declarations: &[(std::path::PathBuf, CompactString)],
    ) {
        if declarations.is_empty() {
            return;
        }
        let snapshots = declarations
            .iter()
            .map(|(path, _)| path.as_path())
            .collect::<vize_carton::FxHashSet<_>>();
        let stubs = self
            .virtual_ts_options
            .auto_import_stubs
            .iter()
            .enumerate()
            .filter_map(|(index, stub)| {
                self.mirrored_reference_directive(stub, &snapshots)
                    .map(|stub| (index, stub))
            })
            .collect::<Vec<_>>();
        for (index, stub) in stubs {
            if let Some(target) = self.virtual_ts_options.auto_import_stubs.get_mut(index) {
                *target = stub;
            }
        }
        for (path, _) in declarations {
            let Some(file) = self.find_by_original(path) else {
                continue;
            };
            let virtual_path = file.virtual_path.clone();
            let mut content = CompactString::with_capacity(file.content.len());
            for line in file.content.split_inclusive('\n') {
                if let Some(reference) = self.mirrored_reference_directive(line, &snapshots) {
                    content.push_str(&reference);
                    if line.ends_with('\n') {
                        content.push('\n');
                    }
                } else {
                    content.push_str(line);
                }
            }
            if let Some(file) = self.virtual_files.get_mut(&virtual_path) {
                file.content = content;
            }
        }
    }

    /// `./relative` for an absolute project specifier whose script module is
    /// mirrored at the same relative path, keeping the authored extension form.
    fn stub_mirror(&self, specifier: &str) -> Option<CompactString> {
        let path = std::path::Path::new(specifier);
        if !path.is_absolute() || specifier.contains("/node_modules/") {
            return None;
        }
        let relative = path.strip_prefix(&self.project_root).ok()?;
        let mirrored = mirror_candidates(specifier)
            .into_iter()
            .any(|(stem, extension)| self.mirrors_at_same_path(stem, extension));
        mirrored.then(|| {
            let mut rendered = CompactString::from("./");
            rendered.push_str(&relative.to_string_lossy().replace('\\', "/"));
            rendered
        })
    }

    /// Whether the project file `{stem}{extension}` exists and is mirrored
    /// at the same path under the virtual root.
    fn mirrors_at_same_path(&self, stem: &str, extension: &str) -> bool {
        let Ok(relative) = std::path::Path::new(stem).strip_prefix(&self.project_root) else {
            return false;
        };
        let mut candidate = std::ffi::OsString::from(stem);
        candidate.push(extension);
        let candidate = std::path::PathBuf::from(candidate);
        let mut virtual_candidate = self.virtual_root.join(relative).into_os_string();
        virtual_candidate.push(extension);
        candidate.is_file()
            && self
                .original_index
                .get(&candidate)
                .is_some_and(|virtual_path| virtual_path.as_os_str() == virtual_candidate)
    }
}

/// The files a specifier can name, as `(stem, extension)`: the path itself,
/// and the TypeScript files TypeScript resolves it to. A `.mjs` or `.cjs`
/// specifier is how a `.mts` or `.cts` module, or its declaration, is
/// imported; an extensionless one also names a `.d.ts`.
fn mirror_candidates(specifier: &str) -> Vec<(&str, &'static str)> {
    if let Some(stem) = specifier.strip_suffix(".mjs") {
        return vec![(specifier, ""), (stem, ".mts"), (stem, ".d.mts")];
    }
    if let Some(stem) = specifier.strip_suffix(".cjs") {
        return vec![(specifier, ""), (stem, ".cts"), (stem, ".d.cts")];
    }
    [
        "",
        ".ts",
        ".tsx",
        ".mts",
        ".d.ts",
        "/index.ts",
        "/index.tsx",
    ]
    .into_iter()
    .map(|extension| (specifier, extension))
    .collect()
}

/// The byte range of the next quoted module specifier at or after `from`:
/// the operand of `import(…)` or of a `from` clause.
fn next_module_specifier(stub: &str, from: usize) -> Option<(usize, usize)> {
    let mut cursor = from;
    loop {
        let rest = stub.get(cursor..)?;
        let open = ["import(", " from "]
            .into_iter()
            .filter_map(|lead| Some(rest.find(lead)? + lead.len()))
            .min()?
            + cursor;
        cursor = open;
        let Some(quote) = stub
            .get(open..open + 1)
            .filter(|quote| *quote == "'" || *quote == "\"")
        else {
            continue;
        };
        let start = open + 1;
        let length = stub.get(start..)?.find(quote)?;
        return Some((start, start + length));
    }
}

fn is_module_augmentation_stub(stub: &str) -> bool {
    stub.starts_with(MODULE_AUGMENTATION_STUB_PREFIX)
}
