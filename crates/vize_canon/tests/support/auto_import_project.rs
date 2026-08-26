//! A throwaway project whose framework auto-imports are declared exactly the
//! way `vize check` declares Nuxt's: as ambient `declare const` stubs that the
//! batch virtual project materializes into one program-wide `.d.ts`.
//!
//! The bundled `vue` declaration mirrors the real package's *branded* `Ref`, so
//! the "a plain `{ value: T }` object is not a ref" control (#3767) is a real
//! control and not an artefact of a structural stub.

use std::path::{Path, PathBuf};

use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait, virtual_ts::VirtualTsOptions,
};

/// A single authored diagnostic: code, 1-based authored line and column, and
/// the complete message. Comparing whole `Vec<Diagnostic>` values keeps every
/// assertion a full-equality one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: Option<u32>,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

pub struct Project {
    dir: tempfile::TempDir,
    options: VirtualTsOptions,
}

impl Project {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("temporary project should be created");
        write_file(dir.path(), "tsconfig.json", TSCONFIG);
        write_file(dir.path(), "node_modules/vue/package.json", VUE_MANIFEST);
        write_file(dir.path(), "node_modules/vue/index.d.ts", VUE_TYPES);
        Self {
            dir,
            options: VirtualTsOptions::default(),
        }
    }

    pub fn write(&mut self, path: &str, source: &str) {
        write_file(self.dir.path(), path, source);
    }

    /// Declare `names` the way a framework auto-import manifest does: an
    /// ambient value binding re-exported from the project's own composables.
    pub fn declare_auto_imports(&mut self, names: &[&str]) {
        let composables = self.dir.path().join("src/composables");
        let module = composables.to_string_lossy().replace('\\', "/");
        for name in names {
            self.options
                .auto_import_stubs
                .push(format!("declare const {name}: typeof import('{module}')['{name}'];").into());
        }
    }

    /// Every diagnostic reported against `path`, sorted by authored line,
    /// column and code.
    pub fn diagnostics(&self, path: &str) -> Vec<Diagnostic> {
        let mut checker = BatchTypeChecker::with_options(
            self.dir.path(),
            BatchTypeCheckerOptions {
                tsconfig_path: Some(self.dir.path().join("tsconfig.json")),
                virtual_ts_options: self.options.clone(),
            },
        )
        .expect("type checker should start");
        checker.scan_project().expect("project should scan");
        let result = checker.check_project().expect("project should type check");
        let mut diagnostics: Vec<Diagnostic> = result
            .diagnostics
            .into_iter()
            // `diagnostic.file` is a `PathBuf`, so this is `Path::ends_with`:
            // a component-wise match that is separator-agnostic.
            .filter(|diagnostic| diagnostic.file.ends_with(path))
            .map(|diagnostic| Diagnostic {
                code: diagnostic.code,
                line: diagnostic.line + 1,
                column: diagnostic.column + 1,
                message: diagnostic.message.to_string(),
            })
            .collect();
        diagnostics.sort_by_key(|diagnostic| (diagnostic.line, diagnostic.column, diagnostic.code));
        diagnostics
    }
}

/// Whether a Corsa/tsgo binary is reachable. Without one the batch checker
/// cannot produce diagnostics, and the test declines rather than passing
/// vacuously.
///
/// Required CI lanes fail closed instead: with `VIZE_TEST_REQUIRE_TSGO` set, a
/// missing toolchain is an assertion failure, so the suite can never report a
/// green run in which no assertion executed. `VIZE_TEST_DISABLE_TSGO` is the
/// explicit opt-out and takes precedence over discovery.
pub fn corsa_available() -> bool {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return false;
    }
    let available = discover_corsa();
    assert!(
        available || std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
        "VIZE_TEST_REQUIRE_TSGO is set, but no tsgo executable was found"
    );
    available
}

fn discover_corsa() -> bool {
    if let Ok(path) = std::env::var("CORSA_PATH")
        && Path::new(&path).exists()
    {
        return true;
    }
    let Some(root) = workspace_root() else {
        return false;
    };
    [
        root.join("node_modules/.bin/tsgo"),
        root.join("tests/node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .iter()
    .any(|candidate| candidate.exists())
        // The checker resolves its own binary this way, so the guard must not
        // decline a toolchain the run would actually have used.
        || vize_s0::corsa_resolver::discover_corsa_in_ancestors(&root).is_some()
}

fn workspace_root() -> Option<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

fn write_file(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent directory should be created");
    }
    std::fs::write(path, source).expect("fixture should be written");
}

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ES2022"
  },
  "include": ["src/**/*"]
}"#;

const VUE_MANIFEST: &str = r#"{ "name": "vue", "types": "index.d.ts" }"#;

/// The subset of `vue`'s public types these fixtures use, keeping the real
/// package's nominal `RefSymbol` brand.
const VUE_TYPES: &str = r#"declare const RefSymbol: unique symbol;
declare const ShallowRefMarker: unique symbol;
export interface Ref<T = any, S = T> {
  get value(): T;
  set value(_: S);
  [RefSymbol]: true;
}
export type ShallowRef<T = any, S = T> = Ref<T, S> & { [ShallowRefMarker]?: true };
export interface WritableComputedRef<T = any, S = T> extends Ref<T, S> {}
export interface ComputedRef<T = any> extends WritableComputedRef<T> {
  readonly value: T;
}
export declare function ref<T>(value: T): Ref<T, T>;
export declare function shallowRef<T>(value: T): ShallowRef<T, T>;
export declare function computed<T>(getter: () => T): ComputedRef<T>;
export declare function computed<T, S = T>(options: {
  get: () => T;
  set: (value: S) => void;
}): WritableComputedRef<T, S>;
export declare function readonly<T>(target: Ref<T, any>): Readonly<Ref<T, T>>;
export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
"#;
