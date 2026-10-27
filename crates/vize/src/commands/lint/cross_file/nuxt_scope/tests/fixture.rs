//! Inert native protocol fixtures only. These packages are NOT a real Nuxt
//! acceptance environment; the separate Actions cohort installs genuine bytes.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use vize_croquis_cf::{
    CrossFileAnalyzer, CrossFileDiagnosticKind, CrossFileOptions, NuxtRenderDirs,
};
use vize_l0::String;

use super::super::super::sfc::analyze_sfc_for_cross_file;
use super::super::admit_with_environment;
use crate::commands::project_nuxt::host_authority::{io::IoCounts, rc::Environment};

pub(super) struct Fixture {
    pub(super) temp: tempfile::TempDir,
    pub(super) root: PathBuf,
    pub(super) home: PathBuf,
    pub(super) sources: Vec<String>,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let base = std::fs::canonicalize(temp.path()).unwrap();
        let root = base.join("project");
        let home = base.join("home");
        std::fs::create_dir_all(home.join(".config")).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        let corpus: Value = serde_json::from_str(
            &std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/_fixtures/cross-file/nuxt-exclusive-id-10919.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let mut fixture = Self {
            temp,
            root,
            home,
            sources: Vec::new(),
        };
        for (name, value) in corpus["originalFiles"].as_object().unwrap() {
            fixture.source(name, value.as_str().unwrap());
        }
        fixture.json(
            "package.json",
            json!({
                "name": "inert-native-scope-protocol-fixture", "version": "0.0.0", "type": "module",
            }),
        );
        for (name, version) in [
            ("nuxt", "4.6.3"),
            ("@nuxt/kit", "4.6.3"),
            ("@nuxt/schema", "4.6.3"),
            ("rc9", "3.1.0"),
        ] {
            fixture.package(&fixture.root.join("node_modules").join(name), name, version);
        }
        fixture
    }

    pub(super) fn source(&mut self, name: &str, source: &str) {
        self.write(name, source.as_bytes());
        if name.ends_with(".vue") && !self.sources.iter().any(|known| known == name) {
            self.sources.push(name.into());
        }
    }

    pub(super) fn write(&self, name: &str, bytes: &[u8]) {
        let path = self.root.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    pub(super) fn json(&self, name: &str, value: Value) {
        self.write(name, &serde_json::to_vec(&value).unwrap());
    }

    pub(super) fn package(&self, root: &Path, name: &str, version: &str) {
        std::fs::create_dir_all(root.join("dist")).unwrap();
        let mut manifest = json!({
            "name": name, "version": version, "type": "module",
            "exports": crate::commands::project_nuxt::host_authority::authority::published_exports(name).unwrap(),
        });
        if let Some(imports) =
            crate::commands::project_nuxt::host_authority::authority::published_imports(name)
        {
            manifest["imports"] = imports;
        }
        if name == "@nuxt/kit" {
            manifest["inlinedDependencies"] = json!({
                "c12": "4.0.0-rc.3", "pkg-types": "2.4.0", "untyped": "2.0.0",
            });
        }
        std::fs::write(
            root.join("package.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("dist/index.mjs"),
            "// INERT NATIVE RESOLUTION FIXTURE; not genuine Nuxt bytes.\nexport {};\n",
        )
        .unwrap();
    }

    pub(super) fn dirs(&self) -> NuxtRenderDirs {
        crate::commands::project_nuxt::render_dirs(&self.root).unwrap()
    }

    pub(super) fn analyzer(&self, omitted: &[&str]) -> CrossFileAnalyzer {
        self.analyzer_erasing(omitted, false)
    }

    pub(super) fn analyzer_erasing(&self, omitted: &[&str], erase: bool) -> CrossFileAnalyzer {
        let mut analyzer = CrossFileAnalyzer::with_project_root(
            CrossFileOptions::minimal().with_unique_ids(true),
            &self.root,
        );
        analyzer.set_nuxt_render_dirs(Some(self.dirs()));
        for name in &self.sources {
            if omitted.iter().any(|omitted| *omitted == name.as_str()) {
                continue;
            }
            let path = self.root.join(name.as_str());
            let source = std::fs::read_to_string(&path).unwrap();
            let mut captured = analyze_sfc_for_cross_file(&source, &path).unwrap();
            if erase {
                // Simulate old/default/partial DTO consumers that do not carry
                // private original-source evidence. Host claims cannot revive it.
                captured.analysis.nuxt_static_render_script = None;
                captured.analysis.nuxt_render_template = None;
            }
            analyzer.add_file_with_analysis_and_effect_summary(
                &path,
                &source,
                captured.analysis,
                captured.effects,
            );
        }
        analyzer.rebuild_import_edges();
        analyzer
    }

    pub(super) fn environment(&self) -> Environment {
        Environment {
            home: Some(self.home.clone()),
            xdg: None,
            opaque: false,
        }
    }

    pub(super) fn prove(&self, analyzer: &mut CrossFileAnalyzer) -> (bool, IoCounts) {
        self.prove_with_environment(analyzer, &self.environment())
    }

    pub(super) fn prove_with_environment(
        &self,
        analyzer: &mut CrossFileAnalyzer,
        environment: &Environment,
    ) -> (bool, IoCounts) {
        let result = admit_with_environment(&self.root, &self.dirs(), analyzer, environment);
        analyzer.admit_nuxt_render_scope_once(result.0);
        result
    }

    pub(super) fn ids(analyzer: &mut CrossFileAnalyzer) -> Vec<String> {
        let mut ids: Vec<_> = analyzer
            .analyze()
            .diagnostics
            .into_iter()
            .filter_map(|diagnostic| match diagnostic.kind {
                CrossFileDiagnosticKind::DuplicateElementId { id, .. } => Some(id.into()),
                _ => None,
            })
            .collect();
        ids.sort();
        ids
    }

    pub(super) fn manifest_bytes(&self) -> u64 {
        [
            "package.json",
            "node_modules/nuxt/package.json",
            "node_modules/@nuxt/kit/package.json",
            "node_modules/@nuxt/schema/package.json",
            "node_modules/rc9/package.json",
        ]
        .into_iter()
        .map(|path| std::fs::metadata(self.root.join(path)).unwrap().len())
        .sum()
    }
}
