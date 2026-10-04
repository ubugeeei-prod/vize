use std::path::{Path, PathBuf};
use vize_carton::ToCompactString;

use super::{ResolutionScope, TypeSourceSnapshot, source_path};
use crate::script::context::external_types::resolution::resolve_import_path;

const EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".d.ts", ".mts", ".cts", ".js", ".jsx", ".vue",
];
const INDEX_NAMES: &[&str] = &[
    "index.ts",
    "index.tsx",
    "index.d.ts",
    "index.mts",
    "index.cts",
    "index.js",
    "index.jsx",
    "index.vue",
];

impl TypeSourceSnapshot {
    pub fn resolve_import(&self, current_file: &Path, specifier: &str) -> Option<PathBuf> {
        let current_file = source_path(current_file);
        // The compatibility package resolver rejects a node_modules component,
        // even when it is the filename. Parentless paths also differ from a
        // normal importer in the root directory. Keep these rare file contexts
        // separate while retaining their positive and negative snapshot cells.
        let scope = match current_file.parent() {
            Some(directory)
                if !current_file
                    .file_name()
                    .is_some_and(|name| name == "node_modules") =>
            {
                ResolutionScope::Directory(directory.to_path_buf())
            }
            _ => ResolutionScope::File(current_file.clone()),
        };
        // This resolver uses only the importer directory and exact specifier,
        // including for package ancestry and @/ aliases. It has no module-mode
        // input; Canon's contextual package route resolver remains separate.
        let key = (scope, specifier.to_compact_string());
        let entry = self
            .resolutions
            .lock()
            .ok()?
            .entry(key)
            .or_default()
            .clone();
        // A shared import resolves once. Filesystem work never holds the map
        // lock, so unrelated directories and specifiers can resolve in parallel.
        entry
            .get_or_init(|| self.resolve_uncached(&current_file, specifier))
            .clone()
    }

    fn resolve_uncached(&self, current_file: &Path, specifier: &str) -> Option<PathBuf> {
        let candidate = if specifier.starts_with('.') {
            Some(current_file.parent()?.join(specifier))
        } else if specifier.starts_with('/') {
            Some(PathBuf::from(specifier))
        } else if let Some(rest) = specifier.strip_prefix("@/") {
            current_file
                .parent()?
                .ancestors()
                .find(|path| path.file_name().is_some_and(|name| name == "src"))
                .map(|src| src.join(rest))
        } else {
            None
        };
        if let Some(candidate) = candidate {
            return self.resolve_candidate(&candidate);
        }
        // Package exports are handled by the existing package resolver. Source
        // reads still go through this snapshot once the module is identified.
        resolve_import_path(current_file, specifier)
    }

    fn exists(&self, candidate: &Path) -> Option<PathBuf> {
        let path = source_path(candidate);
        (self.overlays.contains_key(&path) || path.is_file()).then_some(path)
    }

    fn resolve_candidate(&self, candidate: &Path) -> Option<PathBuf> {
        // Match the compatibility resolver's JS-to-TypeScript preference.
        let replacements: &[&str] = match candidate.extension().and_then(|ext| ext.to_str()) {
            Some("js") => &["ts", "tsx", "d.ts"],
            Some("jsx") => &["tsx", "ts", "d.ts"],
            Some("mjs") => &["mts", "d.mts", "ts", "d.ts"],
            Some("cjs") => &["cts", "d.cts", "ts", "d.ts"],
            _ => &[],
        };
        for extension in replacements {
            if let Some(path) = self.exists(&candidate.with_extension(extension)) {
                return Some(path);
            }
        }
        if let Some(path) = self.exists(candidate) {
            return Some(path);
        }
        for extension in EXTENSIONS {
            let mut path = candidate.as_os_str().to_os_string();
            path.push(extension);
            if let Some(path) = self.exists(Path::new(&path)) {
                return Some(path);
            }
        }
        for name in INDEX_NAMES {
            if let Some(path) = self.exists(&candidate.join(name)) {
                return Some(path);
            }
        }
        None
    }
}
