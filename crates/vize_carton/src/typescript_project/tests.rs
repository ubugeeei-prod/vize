use super::{
    CompilerConfigHost, CompilerOptionsContext, normalize_path_lexically, parse_jsonc_value,
};
use crate::{FxHashMap, String, cstr};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

struct MemoryHost {
    files: FxHashMap<PathBuf, &'static str>,
    reads: RefCell<FxHashMap<PathBuf, usize>>,
}
impl MemoryHost {
    fn new(files: &[(&str, &'static str)]) -> Self {
        Self {
            files: files
                .iter()
                .map(|(path, text)| (PathBuf::from(path), *text))
                .collect(),
            reads: RefCell::default(),
        }
    }
}
impl CompilerConfigHost for MemoryHost {
    type Error = String;
    fn read_jsonc(&self, path: &Path) -> Result<Value, String> {
        *self.reads.borrow_mut().entry(path.to_owned()).or_default() += 1;
        let text = self
            .files
            .get(path)
            .ok_or_else(|| cstr!("missing {}", path.display()))?;
        parse_jsonc_value(text).map_err(|error| cstr!("{error}"))
    }
    fn configuration_error(&self, message: String) -> String {
        message
    }
    fn resolve_extends(&self, path: &Path, specifier: &str) -> Option<PathBuf> {
        let candidate = normalize_path_lexically(&path.parent()?.join(specifier));
        self.files.contains_key(&candidate).then_some(candidate)
    }
}

#[test]
fn shared_chain_is_linear_over_diamonds_and_keeps_ordered_targets() {
    let host = MemoryHost::new(&[
        (
            "/project/base.json",
            r#"{"compilerOptions":{"paths":{"@/*":["first/*","second/*"]},"strict":true}}"#,
        ),
        (
            "/project/left.json",
            r#"{"extends":"base.json","compilerOptions":{"strict":false}}"#,
        ),
        (
            "/project/right.json",
            r#"{"extends":"base.json","compilerOptions":{"noEmit":true}}"#,
        ),
        (
            "/project/app/tsconfig.json",
            r#"{"extends":["../left.json","../right.json"],"compilerOptions":{"baseUrl":"src"}}"#,
        ),
    ]);
    let result = CompilerOptionsContext::new(Path::new("/project/app"))
        .load_flattened(Some(Path::new("/project/app/tsconfig.json")), &host)
        .unwrap();
    assert_eq!(result.options["strict"], true);
    assert_eq!(result.base_url.as_deref(), Some("src"));
    assert_eq!(
        result.options["paths"]["@/*"],
        json!(["src/first/*", "src/second/*"])
    );
    assert_eq!(result.input_paths.len(), 4);
    assert!(result.unresolved_extends.is_empty());
    assert_eq!(host.reads.borrow()[Path::new("/project/base.json")], 1);
    assert!(host.reads.borrow().values().all(|count| *count == 1));
}

#[test]
fn inherited_base_url_owns_overriding_paths_and_verbatim_stays_authored() {
    let host = MemoryHost::new(&[
        (
            "/project/base.json",
            r#"{"compilerOptions":{"baseUrl":"lib"}}"#,
        ),
        (
            "/project/app/tsconfig.json",
            r#"{"extends":"../base.json","compilerOptions":{"paths":{"@/*":["local/*"]}}}"#,
        ),
    ]);
    let context = CompilerOptionsContext::new(Path::new("/project/app"));
    let flattened = context
        .load_flattened(Some(Path::new("/project/app/tsconfig.json")), &host)
        .unwrap();
    assert_eq!(flattened.base_url.as_deref(), Some("/project/lib"));
    assert_eq!(
        flattened.options["paths"]["@/*"],
        json!(["/project/lib/local/*"])
    );
    let authored = context
        .load_verbatim(Path::new("/project/app/tsconfig.json"), &host)
        .unwrap();
    assert_eq!(authored["baseUrl"], "lib");
    assert_eq!(authored["paths"]["@/*"], json!(["local/*"]));
}

#[test]
fn selected_jsconfig_defaults_and_unresolved_parents_remain_explicit() {
    let host = MemoryHost::new(&[(
        "/project/jsconfig.json",
        r#"{"extends":"missing.json","compilerOptions":{"noEmit":false}}"#,
    )]);
    let result = CompilerOptionsContext::observed(Path::new("/project"))
        .load_flattened(Some(Path::new("/project/jsconfig.json")), &host)
        .unwrap();
    assert_eq!(result.options["allowJs"], true);
    assert_eq!(result.options["maxNodeModuleJsDepth"], 2);
    assert_eq!(result.options["noEmit"], false);
    assert_eq!(result.unresolved_extends.len(), 1);
    assert_eq!(result.unresolved_extends[0].specifier, "missing.json");
}

#[test]
fn strict_admission_keeps_malformed_and_cycle_failures_distinct_from_absence() {
    let host = MemoryHost::new(&[
        ("/project/invalid.json", r#"{"extends":[17]}"#),
        ("/project/cycle.json", r#"{"extends":"cycle.json"}"#),
        ("/project/bad.json", "{"),
    ]);
    let context = CompilerOptionsContext::observed(Path::new("/project"));
    assert_eq!(
        context
            .load_flattened(Some(Path::new("/project/invalid.json")), &host)
            .unwrap_err(),
        "cannot admit tsconfig /project/invalid.json: extends must be a string or an array of strings"
    );
    assert_eq!(
        context
            .load_flattened(Some(Path::new("/project/cycle.json")), &host)
            .unwrap_err(),
        "compiler configuration cycle includes /project/cycle.json"
    );
    assert!(
        context
            .load_flattened(Some(Path::new("/project/bad.json")), &host)
            .is_err()
    );
    let before = host.reads.borrow().clone();
    let absent = context.load_flattened(None, &host).unwrap();
    assert!(absent.options.is_empty());
    assert_eq!(*host.reads.borrow(), before);
}

#[test]
fn default_extends_resolver_observes_package_selection_with_a_pure_reader() {
    struct ObservedFiles(RefCell<Vec<PathBuf>>);
    impl CompilerConfigHost for ObservedFiles {
        type Error = String;
        fn read_jsonc(&self, path: &Path) -> Result<Value, String> {
            let text = std::fs::read_to_string(path).map_err(|error| cstr!("{error}"))?;
            parse_jsonc_value(&text).map_err(|error| cstr!("{error}"))
        }
        fn configuration_error(&self, message: String) -> String {
            message
        }
        fn observe(&self, path: &Path) {
            self.0.borrow_mut().push(path.to_path_buf());
        }
    }
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let project = root.join("app");
    let package = root.join("node_modules/@scope/config");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        project.join("tsconfig.json"),
        r#"{"extends":"@scope/config"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"tsconfig":"selected.json"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("selected.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )
    .unwrap();
    let host = ObservedFiles(RefCell::default());
    let selected = project.join("tsconfig.json");
    let options = CompilerOptionsContext::observed(&project)
        .load_flattened(Some(&selected), &host)
        .unwrap();
    assert_eq!(options.options["strict"], true);
    let mut observations = host.0.borrow().clone();
    observations.sort();
    observations.dedup();
    let mut expected = vec![
        selected.clone(),
        project.join("node_modules/@scope/config/package.json"),
        project.join("node_modules/@scope/config/tsconfig.json"),
        package.join("package.json"),
        package.join("selected.json"),
    ];
    expected.sort();
    assert_eq!(observations, expected);
    std::fs::write(
        package.join("package.json"),
        r#"{"tsconfig":"replacement.json"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("replacement.json"),
        r#"{"compilerOptions":{"strict":false}}"#,
    )
    .unwrap();
    let replaced = CompilerOptionsContext::observed(&project)
        .load_flattened(Some(&selected), &host)
        .unwrap();
    assert_eq!(replaced.options["strict"], false);
    let nearer = project.join("node_modules/@scope/config");
    std::fs::create_dir_all(&nearer).unwrap();
    std::fs::write(nearer.join("package.json"), r#"{"tsconfig":"nearer.json"}"#).unwrap();
    std::fs::write(
        nearer.join("nearer.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )
    .unwrap();
    let nearer_options = CompilerOptionsContext::observed(&project)
        .load_flattened(Some(&selected), &host)
        .unwrap();
    assert_eq!(nearer_options.options["strict"], true);
    let mut expected_inputs = vec![selected, nearer.join("nearer.json")];
    expected_inputs.sort();
    assert_eq!(nearer_options.input_paths, expected_inputs);
}
