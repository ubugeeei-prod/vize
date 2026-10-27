//! Runtime authority from bounded importer-scoped Node package winners.
//! No route-source discovery, global witness cache, or package-code parsing.

use serde_json::Value;
use std::path::Path;
use vize_l0::String;

use super::io::{ManifestEvidence, PackageDirectory, PathMetadata, ProbeIo};

const LOOKUP_DEPTH: usize = 64;

#[derive(Debug)]
pub(crate) struct RuntimeAuthority {
    pub(crate) project: ManifestEvidence,
    pub(crate) nuxt: ManifestEvidence,
    pub(crate) kit: ManifestEvidence,
    pub(crate) schema: ManifestEvidence,
    pub(crate) core_schema: ManifestEvidence,
    pub(crate) rc9: ManifestEvidence,
}

impl RuntimeAuthority {
    pub(crate) fn read(root: &Path, io: &mut ProbeIo) -> Option<Self> {
        let project = io.manifest(root)?;
        if project.has_imports
            || project.manager_overrides
            || project
                .name
                .as_deref()
                .is_some_and(|name| matches!(name, "nuxt" | "@nuxt/kit" | "@nuxt/schema" | "rc9"))
        {
            return None;
        }
        let nuxt = package_winner(root, "nuxt", io)?;
        if !reviewed(&nuxt, "nuxt", "4.6.3", io) {
            return None;
        }
        // Resolve every dependency from its actual importing physical package.
        // A convenient top-level copy does not identify kit's loader inputs.
        let kit = package_winner(&nuxt.package_root.join("dist"), "@nuxt/kit", io)?;
        if !reviewed(&kit, "@nuxt/kit", "4.6.3", io) {
            return None;
        }
        let schema = package_winner(&kit.package_root.join("dist"), "@nuxt/schema", io)?;
        let rc9 = package_winner(&kit.package_root.join("dist"), "rc9", io)?;
        let core_schema = package_winner(&nuxt.package_root.join("dist"), "@nuxt/schema", io)?;
        if !reviewed(&schema, "@nuxt/schema", "4.6.3", io)
            || !reviewed(&core_schema, "@nuxt/schema", "4.6.3", io)
            || !reviewed(&rc9, "rc9", "3.1.0", io)
        {
            return None;
        }

        // kit 4.6.3 bundles its loader. Requiring an imaginary installed c12
        // would authenticate a different package. Its published manifest
        // identifies the reviewed bundled loader and workspace-root reader.
        if kit.inlined_dependencies.len() != 3
            || kit.inlined_dependencies.get("c12").map(String::as_str) != Some("4.0.0-rc.3")
            || kit
                .inlined_dependencies
                .get("pkg-types")
                .map(String::as_str)
                != Some("2.4.0")
            || kit.inlined_dependencies.get("untyped").map(String::as_str) != Some("2.0.0")
        {
            return None;
        }
        Some(Self {
            project,
            nuxt,
            kit,
            schema,
            core_schema,
            rc9,
        })
    }

    pub(crate) fn witness_bytes(&self) -> u64 {
        let mut paths = vize_l0::FxHashSet::default();
        [
            &self.project,
            &self.nuxt,
            &self.kit,
            &self.schema,
            &self.core_schema,
            &self.rc9,
        ]
        .into_iter()
        .filter(|manifest| paths.insert(manifest.package_root.as_path()))
        .map(|manifest| manifest.bytes)
        .sum()
    }
}

fn reviewed(manifest: &ManifestEvidence, name: &str, version: &str, io: &mut ProbeIo) -> bool {
    if manifest.name.as_deref() != Some(name)
        || manifest.version.as_deref() != Some(version)
        || !manifest.module
        || manifest.imports != published_imports(name)
    {
        return false;
    }
    if manifest.exports != published_exports(name) {
        return false;
    }
    // Node resolves bare imports from the physical exported module, not its
    // manifest directory. A linked dist directory would change that context.
    let dist = manifest.package_root.join("dist");
    if !matches!(
        io.metadata(&dist.join("package.json")),
        PathMetadata::Missing
    ) {
        // A nearer package scope can add self references or change module type.
        return false;
    }
    io.canonical_directory(&dist, false)
        .is_some_and(|physical| physical == dist)
        && io.regular_file(&dist.join("index.mjs"))
}

pub(crate) fn published_imports(name: &str) -> Option<Value> {
    (name == "nuxt").then(|| {
        serde_json::json!({
            "#app": "./dist/app/index.js",
            "#app/nuxt": "./dist/app/nuxt.js",
            "#pages/composables": "./dist/pages/runtime/composables.js",
            "#unhead/composables": "./dist/head/runtime/composables.js"
        })
    })
}

pub(crate) fn published_exports(name: &str) -> Option<Value> {
    Some(match name {
        "nuxt" => serde_json::json!({
            ".": {
                "types": "./types.d.mts",
                "import": "./dist/index.mjs"
            },
            "./app": "./dist/app/index.js",
            "./kit": "./kit.js",
            "./meta": "./meta.js",
            "./entry": "./dist/runtime/server/entry.js",
            "./config": {
                "types": "./config.d.ts",
                "import": "./config.js",
                "require": "./config.js"
            },
            "./schema": "./schema.js",
            "./server": "./dist/server/index.js",
            "./styles": "./dist/runtime/server/styles.js",
            "./manifest": "./dist/runtime/server/manifest.js",
            "./entry-ids": "./dist/runtime/server/entry-ids.js",
            "./internal/*": "./dist/runtime/server/*.js",
            "./entry-chunk": "./dist/runtime/server/entry-chunk.js",
            "./precomputed": "./dist/runtime/server/precomputed.js",
            "./package.json": "./package.json",
            "./internal/renderer": "./dist/runtime/server/renderer/index.js",
            "./internal/dev-error": "./dist/runtime/server/dev-error/index.js",
            "./internal/dev-error/sourcemap": "./dist/runtime/server/dev-error/sourcemap.js"
        }),
        "@nuxt/kit" => serde_json::json!({
            ".": "./dist/index.mjs",
            "./internal": "./dist/internal/index.mjs",
            "./package.json": "./package.json"
        }),
        "@nuxt/schema" => serde_json::json!({
            ".": "./dist/index.mjs",
            "./internal": "./dist/internal.mjs",
            "./builder-env": "./dist/builder-env.mjs",
            "./package.json": "./package.json"
        }),
        "rc9" => serde_json::json!({
            ".": "./dist/index.mjs"
        }),
        _ => return None,
    })
}

/// Bounded node_modules branch for a policy-authenticated package scope.
/// Callers must first refuse nearer self references/import aliases and interpret
/// the actual ESM export condition; this is not a generic Node resolver.
pub(crate) fn package_winner(
    from: &Path,
    package: &str,
    io: &mut ProbeIo,
) -> Option<ManifestEvidence> {
    for directory in from.ancestors().take(LOOKUP_DEPTH) {
        // In this supported chain the nearest package is already known and
        // has a different name. A project self-reference was refused above.
        if directory
            .file_name()
            .is_some_and(|name| name == "node_modules")
        {
            continue;
        }
        let candidate = directory.join("node_modules").join(package);
        io.package_lookup_request();
        match io.package_directory(&candidate) {
            PackageDirectory::Missing => {}
            PackageDirectory::Canonical(root) => return io.manifest(&root),
            PackageDirectory::Unknown => return None,
        }
    }
    None
}
