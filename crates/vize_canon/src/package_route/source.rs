//! TypeScript-family source probing for package route targets.

use std::path::{Path, PathBuf};

use vize_carton::cstr;

use super::{PackageSourceOptions, canonical_path};

const TS_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".vue", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts",
];
const JSX_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".vue", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts", ".jsx",
];
const JS_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".vue", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts", ".js", ".jsx", ".mjs",
    ".cjs",
];

pub(super) fn resolve_source(base: &Path, options: PackageSourceOptions) -> Option<PathBuf> {
    // A declaration sidecar stands in for its runtime module even when that
    // module is absent, so probe it before requiring `base` on disk.
    if let Some(sidecar) = declaration_sidecar(base) {
        return Some(canonical_path(&sidecar));
    }
    if base.is_file() && accepted_source(base, options) {
        return Some(canonical_path(base));
    }
    // A manifest may name the built runtime target (`./dist/index.js`) that a
    // source checkout never emits; its authored twin carries a TypeScript
    // extension in the same place, so swap the extension before appending one.
    if let Some(twin) = typescript_twin(base) {
        return Some(canonical_path(&twin));
    }
    for extension in source_extensions(options) {
        let candidate = append_extension(base, extension);
        if candidate.is_file() {
            return Some(canonical_path(&candidate));
        }
    }
    for extension in source_extensions(options) {
        let candidate = base.join(cstr!("index{extension}").as_str());
        if candidate.is_file() {
            return Some(canonical_path(&candidate));
        }
    }
    None
}

fn source_extensions(options: PackageSourceOptions) -> &'static [&'static str] {
    if options.include_javascript {
        JS_EXTENSIONS
    } else if options.include_jsx {
        JSX_EXTENSIONS
    } else {
        TS_EXTENSIONS
    }
}

fn accepted_source(path: &Path, options: PackageSourceOptions) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    TS_EXTENSIONS
        .iter()
        .any(|extension| name.ends_with(extension))
        || options.include_javascript
            && [".js", ".jsx", ".mjs", ".cjs"]
                .iter()
                .any(|extension| name.ends_with(extension))
        || options.include_jsx && name.ends_with(".jsx")
}

fn declaration_sidecar(path: &Path) -> Option<PathBuf> {
    let extensions: &[&str] = match path.extension().and_then(|ext| ext.to_str()) {
        Some("mjs") => &["d.mts", "d.ts"],
        Some("cjs") => &["d.cts", "d.ts"],
        Some("js" | "jsx") => &["d.ts"],
        _ => &[],
    };
    extensions
        .iter()
        .map(|extension| path.with_extension(extension))
        .find(|candidate| candidate.is_file())
}

/// The authored TypeScript-family file a runtime target was emitted from.
fn typescript_twin(path: &Path) -> Option<PathBuf> {
    let extensions: &[&str] = match path.extension().and_then(|ext| ext.to_str()) {
        Some("js") => &["ts", "tsx", "vue"],
        Some("jsx") => &["tsx", "ts"],
        Some("mjs") => &["mts"],
        Some("cjs") => &["cts"],
        _ => &[],
    };
    extensions
        .iter()
        .map(|extension| path.with_extension(extension))
        .find(|candidate| candidate.is_file())
}

fn append_extension(base: &Path, extension: &str) -> PathBuf {
    base.file_name().and_then(|name| name.to_str()).map_or_else(
        || base.to_path_buf(),
        |name| base.with_file_name(cstr!("{name}{extension}").as_str()),
    )
}
