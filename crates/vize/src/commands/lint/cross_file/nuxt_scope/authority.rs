//! Runtime authority from bounded importer-scoped Node package winners.
//! No route-source discovery, global witness cache, or package content parsing.

use serde_json::Value;
use std::path::Path;
use vize_l0::String;

use super::io::{ManifestEvidence, PackageDirectory, ProbeIo};

const LOOKUP_DEPTH: usize = 64;

#[derive(Debug)]
pub(super) struct RuntimeAuthority {
    pub(super) project: ManifestEvidence,
    pub(super) nuxt: ManifestEvidence,
    pub(super) kit: ManifestEvidence,
    pub(super) schema: ManifestEvidence,
    pub(super) core_schema: ManifestEvidence,
    pub(super) rc9: ManifestEvidence,
}

impl RuntimeAuthority {
    pub(super) fn read(root: &Path, io: &mut ProbeIo) -> Option<Self> {
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
        let kit = package_winner(&nuxt.package_root, "@nuxt/kit", io)?;
        if !reviewed(&kit, "@nuxt/kit", "4.6.3", io) {
            return None;
        }
        let schema = package_winner(&kit.package_root, "@nuxt/schema", io)?;
        let rc9 = package_winner(&kit.package_root, "rc9", io)?;
        let core_schema = package_winner(&nuxt.package_root, "@nuxt/schema", io)?;
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

    pub(super) fn witness_bytes(&self) -> u64 {
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
        || manifest.has_imports
    {
        return false;
    }
    let Some(root) = manifest
        .exports
        .as_ref()
        .and_then(|exports| exports.get("."))
    else {
        return false;
    };
    let supported = if name == "nuxt" {
        root.as_object().is_some_and(|conditions| {
            conditions.len() == 2
                && conditions.get("import").and_then(Value::as_str) == Some("./dist/index.mjs")
                && conditions.get("types").and_then(Value::as_str) == Some("./types.d.mts")
        })
    } else {
        root.as_str() == Some("./dist/index.mjs")
    };
    supported && io.regular_file(&manifest.package_root.join("dist/index.mjs"))
}

fn package_winner(from: &Path, package: &str, io: &mut ProbeIo) -> Option<ManifestEvidence> {
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
