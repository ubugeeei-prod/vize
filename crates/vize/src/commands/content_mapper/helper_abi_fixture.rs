//! Explicit lowering modes and closed filesystem seeds for SDK admission tests.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_canon::{
    ContentMapperProjectConfiguration, generate_vue_content_mapper_transform_with_project_context,
};
use vize_l0::String as CompactString;

use super::test_support::{
    exchange, frames, initialize_request, open_project_request, transform_request,
};

const SOURCE: &str = "<script setup lang=\"ts\">defineProps<{ benchCount: number }>();</script><template><div /></template>";

#[derive(Clone, Copy)]
pub(super) enum Mode {
    Inline,
    Core,
    Shared,
}

pub(super) struct InstalledSdk {
    pub _directory: tempfile::TempDir,
    pub config: PathBuf,
    pub package: PathBuf,
}

impl InstalledSdk {
    pub fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path().join("app");
        let package = directory.path().join("node_modules/vize");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&package).unwrap();
        let config = project.join("tsconfig.json");
        std::fs::write(&config, "{}").unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../npm/cli");
        copy_declarations(&source, &package);
        std::fs::copy(source.join("package.json"), package.join("package.json")).unwrap();
        let vue = directory.path().join("node_modules/vue");
        std::fs::create_dir_all(&vue).unwrap();
        std::fs::write(vue.join("package.json"), "{}").unwrap();
        Self {
            _directory: directory,
            config,
            package,
        }
    }

    pub fn open_result(&self) -> Value {
        self.open_result_with_options(json!({}))
    }

    pub fn open_result_with_options(&self, compiler_options: Value) -> Value {
        let mut open = open_project_request(2, Value::Null, compiler_options);
        open["params"]["configFileName"] = json!(self.config);
        exchange(&frames(&[initialize_request(), open]))[1]["result"].clone()
    }

    pub fn assert_projection(&self, mode: Mode) {
        self.assert_projection_with_options(mode, json!({}));
    }

    pub fn assert_projection_with_options(&self, mode: Mode, compiler_options: Value) {
        self.assert_projection_of(SOURCE, mode, compiler_options);
    }

    pub fn assert_projection_of(&self, source: &str, mode: Mode, compiler_options: Value) {
        let (configuration, _) = ContentMapperProjectConfiguration::load_with_compiler_options(
            &self.config,
            compiler_options.as_object().unwrap(),
        )
        .unwrap();
        let core = matches!(mode, Mode::Core | Mode::Shared);
        let options = configuration
            .transform_options
            .with_external_component_attrs_module(core)
            .with_external_component_type_helpers(core)
            .with_shared_virtual_module(matches!(mode, Mode::Shared));
        let expected = generate_vue_content_mapper_transform_with_project_context(
            Path::new("/project/Options.vue"),
            source,
            options,
            &configuration.project_context,
        )
        .unwrap();
        let mut open = open_project_request(2, Value::Null, compiler_options);
        open["params"]["configFileName"] = json!(self.config);
        let responses = exchange(&frames(&[
            initialize_request(),
            open,
            transform_request(3, source),
        ]));
        assert_eq!(
            responses[2],
            json!({ "id": 3, "jsonrpc": "2.0", "result": super::dispatch::wire_value(&expected) })
        );
    }

    /// The fixture declares the winners; this oracle never calls installed_helpers.
    pub fn expected_watches(
        &self,
        sdk_present: bool,
        helper_vue: Option<&Path>,
        aliases: bool,
    ) -> Value {
        let root = self._directory.path();
        let project = self.config.parent().unwrap();
        let mut paths = BTreeSet::from([self.config.clone()]);
        if aliases {
            let config = std::fs::canonicalize(&self.config).unwrap();
            let base = config.parent().unwrap().join(".");
            for prefix in ["types/vize", "vize"] {
                for module in [
                    "content-mapper-helpers",
                    "virtual-ts",
                    "virtual-ts-checker",
                    "virtual-ts-checker-strict",
                ] {
                    let candidate = base.join(prefix).join(module);
                    paths.insert(candidate.parent().unwrap().to_path_buf());
                    paths.insert(candidate);
                }
            }
        }
        let manifest = self.package.join("package.json");
        lookup_seed(
            &mut paths,
            project,
            "vize",
            sdk_present.then_some(manifest.as_path()),
        );
        if sdk_present {
            let contract: Value = serde_json::from_str(include_str!("helper-abi.json")).unwrap();
            for module in contract["modules"].as_array().unwrap() {
                for declaration in module["files"].as_array().unwrap() {
                    paths.insert(self.package.join(declaration["path"].as_str().unwrap()));
                }
            }
            let project_vue = root.join("node_modules/vue/package.json");
            lookup_seed(&mut paths, project, "vue", Some(&project_vue));
            let physical_sdk = std::fs::canonicalize(&self.package).unwrap();
            let default_vue = std::fs::canonicalize(root)
                .unwrap()
                .join("node_modules/vue/package.json");
            lookup_seed(
                &mut paths,
                &physical_sdk,
                "vue",
                Some(helper_vue.unwrap_or(&default_vue)),
            );
        }
        let mut paths: Vec<CompactString> = paths
            .into_iter()
            .map(|path| CompactString::from(path.to_string_lossy().as_ref()))
            .collect();
        paths.sort();
        json!(paths)
    }

    pub fn rename(&self, file: &str, before: &str, after: &str) {
        let file = self.package.join(file);
        let source = std::fs::read_to_string(&file).unwrap();
        assert_eq!(source.matches(before).count(), 1, "{}", file.display());
        std::fs::write(file, source.replacen(before, after, 1)).unwrap();
    }

    #[cfg(unix)]
    pub fn move_to_pnpm_store(&self, same_vue: bool) {
        let root = self._directory.path();
        let physical = root.join("store/vize/node_modules/vize");
        std::fs::create_dir_all(physical.parent().unwrap()).unwrap();
        std::fs::rename(&self.package, &physical).unwrap();
        std::os::unix::fs::symlink(&physical, &self.package).unwrap();
        let helper_vue = physical.parent().unwrap().join("vue");
        if same_vue {
            std::os::unix::fs::symlink(root.join("node_modules/vue"), helper_vue).unwrap();
        } else {
            std::fs::create_dir_all(&helper_vue).unwrap();
            std::fs::write(helper_vue.join("package.json"), "{}").unwrap();
        }
    }
}

fn lookup_seed(paths: &mut BTreeSet<PathBuf>, from: &Path, package: &str, selected: Option<&Path>) {
    for directory in from.ancestors() {
        let candidate = directory
            .join("node_modules")
            .join(package)
            .join("package.json");
        paths.insert(candidate.clone());
        if selected == Some(candidate.as_path()) {
            paths.insert(std::fs::canonicalize(candidate).unwrap());
            return;
        }
    }
    assert_eq!(
        selected, None,
        "declared fixture winner must be in its ancestor chain"
    );
}

fn copy_declarations(source: &Path, target: &Path) {
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        if entry.file_type().unwrap().is_dir() {
            if name == "generated" {
                std::fs::create_dir_all(target.join(&name)).unwrap();
                copy_declarations(&entry.path(), &target.join(&name));
            }
        } else if name.to_str().is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        }) {
            std::fs::copy(entry.path(), target.join(name)).unwrap();
        }
    }
}
