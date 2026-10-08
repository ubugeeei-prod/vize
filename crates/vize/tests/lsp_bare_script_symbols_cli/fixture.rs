use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

use super::{TestResult, check, corsa_requirement};
use super::{lsp_process::file_uri, reads, session::Session};

const NATIVE_INVALID: &str = "/*😀*/ export const nativeGuard: number = 'wrong';\n";
const VUE_INVALID: &str = "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
const CONFIG: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/bare-script-symbol-routes/tsconfig.json"
);

pub(super) struct Fixture {
    project: tempfile::TempDir,
    original: Vec<(PathBuf, Vec<u8>)>,
    library: PathBuf,
    library_bytes: Vec<u8>,
    runtime: PathBuf,
    pub(super) vize: Session,
    pub(super) native: Session,
}

impl Fixture {
    pub(super) fn new(files: &[(&str, String)], jsx: bool) -> TestResult<Self> {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or("workspace root missing")?;
        let runtime = corsa_requirement::required_or_skip(discover_corsa_in_ancestors(workspace))
            .ok_or("actual frozen native runtime required")?;
        let library = runtime
            .ancestors()
            .flat_map(|ancestor| {
                [
                    ancestor.join("lib.dom.d.ts"),
                    ancestor.join("lib/lib.dom.d.ts"),
                ]
            })
            .find(|path| path.is_file())
            .ok_or("actual bundled DOM library required")?;
        let library_bytes = std::fs::read(&library)?;
        let project = tempfile::tempdir()?;
        let mut original = Vec::new();
        for (file, source) in files {
            let path = project.path().join(file);
            std::fs::create_dir_all(path.parent().ok_or("fixture file parent missing")?)?;
            std::fs::write(&path, source)?;
            original.push((path, source.as_bytes().to_vec()));
        }
        std::fs::create_dir_all(project.path().join("src"))?;
        for (file, source) in [
            ("src/NativeGuard.ts", NATIVE_INVALID),
            ("src/NativeGuard.vue", VUE_INVALID),
            (
                "src/jsx.d.ts",
                "declare namespace JSX { interface IntrinsicElements { i: {} } }\n",
            ),
            ("tsconfig.json", CONFIG),
        ] {
            let path = project.path().join(file);
            std::fs::write(&path, source)?;
            original.push((path, source.as_bytes().to_vec()));
        }
        let modules = project.path().join("node_modules");
        std::fs::create_dir_all(&modules)?;
        let vue = workspace
            .join("playground/node_modules/vue")
            .canonicalize()?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(vue, modules.join("vue"))?;
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(vue, modules.join("vue"))?;
        let config = serde_json::to_vec(&json!({
            "typeChecker":{"corsaPath":runtime,"jsxTypecheck":jsx,"checkFallthroughAttrs":false},
            "lsp":{"lint":false,"typecheck":true,"hover":true,"crossFile":true}
        }))?;
        let path = project.path().join("vize.config.json");
        std::fs::write(&path, &config)?;
        original.push((path, config));
        let native = Session::new(project.path(), Some(&runtime))?;
        let vize = Session::new(project.path(), None)?;
        Ok(Self {
            project,
            original,
            library,
            library_bytes,
            runtime,
            native,
            vize,
        })
    }

    pub(super) fn uri(&self, file: &str) -> String {
        file_uri(&self.project.path().join(file)).to_string()
    }

    pub(super) fn open_both(&mut self, file: &str, source: &str, language: &str, version: i64) {
        let uri = self.uri(file);
        self.native.open(&uri, source, language, version);
        self.vize.open(&uri, source, language, version);
    }

    pub(super) fn change_both(&mut self, file: &str, source: &str, version: i64) {
        let uri = self.uri(file);
        self.native.change(&uri, source, version);
        self.vize.change(&uri, source, version);
    }

    pub(super) fn reopen_both(&mut self, file: &str, language: &str, version: i64) -> TestResult {
        let uri = self.uri(file);
        self.native.close(&uri);
        self.vize.close(&uri);
        let source = std::fs::read_to_string(self.project.path().join(file))?;
        self.open_both(file, &source, language, version);
        Ok(())
    }

    pub(super) fn refuse_writes(
        &mut self,
        file: &str,
        token: reads::Token,
        replacement: &str,
    ) -> TestResult {
        let uri = self.uri(file);
        let position = json!({"line":token.0,"character":token.1});
        for method in ["textDocument/prepareRename", "textDocument/rename"] {
            check(
                self.vize.request(
                    method,
                    &uri,
                    json!({"position":position,"newName":replacement}),
                )?,
                Value::Null,
            )?;
        }
        self.assert_disks()
    }

    pub(super) fn prove_native_diagnostics(&mut self) -> TestResult {
        let native_uri = self.uri("src/NativeGuard.ts");
        self.native
            .open(&native_uri, NATIVE_INVALID, "typescript", 1);
        let error = json!({"range":reads::range((0,20,31)),"severity":1,"code":2322,
            "source":"ts","message":"Type 'string' is not assignable to type 'number'."});
        let bad = json!({"kind":"full","items":[error]});
        check(
            self.native
                .request("textDocument/diagnostic", &native_uri, json!({}))?,
            bad.clone(),
        )?;
        self.native
            .change(&native_uri, &NATIVE_INVALID.replace("'wrong'", "1"), 2);
        check(
            self.native
                .request("textDocument/diagnostic", &native_uri, json!({}))?,
            json!({"kind":"full","items":[]}),
        )?;
        self.native.change(&native_uri, NATIVE_INVALID, 3);
        check(
            self.native
                .request("textDocument/diagnostic", &native_uri, json!({}))?,
            bad,
        )?;
        // Vize's supported Vue route is the independent native activation
        // witness. This deliberately makes no bare diagnostics capability claim.
        let vue_uri = self.uri("src/NativeGuard.vue");
        let error = json!([{"range":reads::range((1,6,17)),"severity":1,"code":2322,
            "source":"vize/types","message":"Type 'string' is not assignable to type 'number'."}]);
        self.vize.open(&vue_uri, VUE_INVALID, "vue", 1);
        self.vize.publication(&vue_uri, 1, error.clone())?;
        self.vize
            .change(&vue_uri, &VUE_INVALID.replace("'wrong'", "1"), 2);
        self.vize.publication(&vue_uri, 2, json!([]))?;
        self.vize.change(&vue_uri, VUE_INVALID, 3);
        self.vize.publication(&vue_uri, 3, error)?;
        self.assert_disks()
    }

    pub(super) fn assert_disks(&self) -> TestResult {
        for (path, expected) in &self.original {
            if std::fs::read(path)?.as_slice() != expected.as_slice() {
                return Err(format!("authored disk bytes changed: {path:?}").into());
            }
        }
        if std::fs::read(&self.library)? != self.library_bytes {
            return Err("actual bundled DOM library bytes changed".into());
        }
        Ok(())
    }

    pub(super) fn finish(&mut self) -> TestResult {
        // Reopen the unchanged diagnostic witnesses after all source reopen
        // transitions; the backend must still expose the complete real error.
        let uri = self.uri("src/NativeGuard.ts");
        self.native.close(&uri);
        self.native.open(&uri, NATIVE_INVALID, "typescript", 1);
        check(
            self.native
                .request("textDocument/diagnostic", &uri, json!({}))?,
            json!({"kind":"full","items":[{"range":reads::range((0,20,31)),
                "severity":1,"code":2322,"source":"ts",
                "message":"Type 'string' is not assignable to type 'number'."}]}),
        )?;
        let uri = self.uri("src/NativeGuard.vue");
        self.vize.close(&uri);
        self.vize.open(&uri, VUE_INVALID, "vue", 1);
        self.vize.publication(
            &uri,
            1,
            json!([{"range":reads::range((1,6,17)),
            "severity":1,"code":2322,"source":"vize/types",
            "message":"Type 'string' is not assignable to type 'number'."}]),
        )?;
        self.native.finish(&self.runtime)?;
        self.vize.finish(&self.runtime)?;
        self.assert_disks()
    }
}
