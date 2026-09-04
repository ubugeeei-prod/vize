//! TypeScript-family source probing for package route targets.

use std::path::{Path, PathBuf};

use vize_carton::cstr;

use super::{PackagePathCache, PackageSourceOptions, canonical_path};

pub(super) struct ResolvedPackageSource {
    pub(super) source_path: PathBuf,
    pub(super) native_probe_path: PathBuf,
}

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

pub(super) fn resolve_sources(
    base: &Path,
    options: PackageSourceOptions,
    consulted: &mut Vec<PathBuf>,
    canonical_paths: &mut PackagePathCache,
) -> Vec<ResolvedPackageSource> {
    let mut resolved = Vec::new();
    // A declaration sidecar stands in for its runtime module even when that
    // module is absent, so probe it before requiring `base` on disk.
    for sidecar in declaration_sidecars(base) {
        record_if_file(
            &sidecar,
            &sidecar,
            consulted,
            &mut resolved,
            canonical_paths,
        );
    }
    consulted.push(canonical_path(base, canonical_paths));
    if base.is_file() && accepted_source(base, options) {
        resolved.push(ResolvedPackageSource {
            source_path: canonical_path(base, canonical_paths),
            native_probe_path: canonical_path(
                &native_probe_for_source(base, base),
                canonical_paths,
            ),
        });
    }
    // A manifest may name the built runtime target (`./dist/index.js`) that a
    // source checkout never emits; its authored twin carries a TypeScript
    // extension in the same place, so swap the extension before appending one.
    for twin in typescript_twins(base) {
        let probe = native_probe_for_source(base, &twin);
        record_if_file(&twin, &probe, consulted, &mut resolved, canonical_paths);
    }
    for extension in source_extensions(options) {
        let candidate = append_extension(base, extension);
        let probe = native_probe_for_source(base, &candidate);
        record_if_file(
            &candidate,
            &probe,
            consulted,
            &mut resolved,
            canonical_paths,
        );
    }
    for extension in source_extensions(options) {
        let candidate = base.join(cstr!("index{extension}").as_str());
        let probe = native_probe_for_source(base, &candidate);
        record_if_file(
            &candidate,
            &probe,
            consulted,
            &mut resolved,
            canonical_paths,
        );
    }
    resolved.sort_by(|left, right| {
        (&left.source_path, &left.native_probe_path)
            .cmp(&(&right.source_path, &right.native_probe_path))
    });
    resolved.dedup_by(|left, right| {
        left.source_path == right.source_path && left.native_probe_path == right.native_probe_path
    });
    resolved
}

fn record_if_file(
    path: &Path,
    native_probe: &Path,
    consulted: &mut Vec<PathBuf>,
    resolved: &mut Vec<ResolvedPackageSource>,
    canonical_paths: &mut PackagePathCache,
) {
    let path = canonical_path(path, canonical_paths);
    consulted.push(path.clone());
    if path.is_file() {
        resolved.push(ResolvedPackageSource {
            source_path: path,
            native_probe_path: canonical_path(native_probe, canonical_paths),
        });
    }
}

fn native_probe_for_source(target: &Path, source: &Path) -> PathBuf {
    if source
        .extension()
        .is_none_or(|extension| extension != "vue")
    {
        return source.to_path_buf();
    }
    match target.extension().and_then(|extension| extension.to_str()) {
        Some("vue") => target.with_extension("d.vue.ts"),
        Some("mjs") => target.with_extension("mts"),
        Some("cjs") => target.with_extension("cts"),
        Some("js" | "jsx") => target.with_extension("ts"),
        _ => source.with_extension("ts"),
    }
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

fn declaration_sidecars(path: &Path) -> Vec<PathBuf> {
    let extensions: &[&str] = match path.extension().and_then(|ext| ext.to_str()) {
        Some("mjs") => &["d.mts", "d.ts"],
        Some("cjs") => &["d.cts", "d.ts"],
        Some("js" | "jsx") => &["d.ts"],
        _ => &[],
    };
    extensions
        .iter()
        .map(|extension| path.with_extension(extension))
        .collect()
}

/// The authored TypeScript-family file a runtime target was emitted from.
fn typescript_twins(path: &Path) -> Vec<PathBuf> {
    let extensions: &[&str] = match path.extension().and_then(|ext| ext.to_str()) {
        Some("js") => &["ts", "tsx", "vue"],
        Some("jsx") => &["tsx", "ts", "vue"],
        Some("mjs") => &["mts", "vue"],
        Some("cjs") => &["cts", "vue"],
        _ => &[],
    };
    extensions
        .iter()
        .map(|extension| path.with_extension(extension))
        .collect()
}

fn append_extension(base: &Path, extension: &str) -> PathBuf {
    base.file_name().and_then(|name| name.to_str()).map_or_else(
        || base.to_path_buf(),
        |name| base.with_file_name(cstr!("{name}{extension}").as_str()),
    )
}
