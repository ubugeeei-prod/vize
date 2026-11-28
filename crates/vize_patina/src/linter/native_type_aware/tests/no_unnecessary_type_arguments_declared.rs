//! `type/no-unnecessary-type-arguments` for generics declared outside the
//! script: imported interfaces, functions, classes and overloads, library
//! packages and `lib.dom.d.ts`, against typescript-eslint v8.72.2.
use super::{corsa_available, failure_context, lint_sfc_with_corsa};
use crate::{LintPreset, Linter, Severity};
use std::path::Path;

const RULE: &str = "type/no-unnecessary-type-arguments";
const MESSAGE: &str = "This is the default value for this type parameter, so it can be omitted.";

const TSCONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unnecessary-type-arguments-declared-elsewhere/tsconfig.json"
));

const TYPES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unnecessary-type-arguments-declared-elsewhere/src/types.ts.txt"
));

const PROJECT_PACKAGE: &str = "{ \"name\": \"declared-defaults\", \"private\": true }\n";

const LIBRARY_PACKAGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unnecessary-type-arguments-declared-elsewhere/generic-library/package.json.txt"
));

const LIBRARY_TYPES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unnecessary-type-arguments-declared-elsewhere/generic-library/index.d.ts.txt"
));

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/no-unnecessary-type-arguments-declared-elsewhere/src/Args.vue.txt"
));

#[test]
fn defaults_declared_outside_the_script_match_typescript_eslint() {
    if !corsa_available() {
        return;
    }

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("no-unnecessary-type-arguments-declared-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    let library = project.path().join("node_modules/generic-library");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::create_dir_all(&library).expect("library package");
    // The project's own manifest roots the session, so its `node_modules`
    // resolves the library package.
    std::fs::write(project.path().join("package.json"), PROJECT_PACKAGE).expect("manifest");
    std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");
    std::fs::write(source_dir.join("types.ts"), TYPES).expect("imported declarations");
    std::fs::write(library.join("package.json"), LIBRARY_PACKAGE).expect("library manifest");
    std::fs::write(library.join("index.d.ts"), LIBRARY_TYPES).expect("library declarations");
    let path = source_dir.join("Args.vue");
    std::fs::write(&path, SOURCE).expect("SFC fixture");

    let linter =
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]));
    let result = lint_sfc_with_corsa(&linter, SOURCE, path.to_str().expect("UTF-8 fixture path"));
    let expected = [
        "Option<unknown>",
        "load<unknown>",
        "Store<string>()",
        "overloaded<unknown>",
        "querySelector<Element>(\"main\");\nconst localValue",
        "Local<unknown>",
        "document.querySelector<Element>",
        "useLibraryRef<unknown>(",
        "useTemplateRef<unknown>(",
        "types.Option<unknown>",
        "extends Store<string>",
    ]
    .map(|needle| {
        let start = SOURCE.find(needle).expect("reported use");
        let open = needle.find('<').expect("type argument list") + 1;
        let close = needle.find('>').expect("type argument list");
        ((start + open) as u32, (start + close) as u32)
    });
    let mut found = result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.start, diagnostic.end))
        .collect::<Vec<_>>();
    found.sort_unstable();
    let mut wanted = expected.to_vec();
    wanted.sort_unstable();
    assert_eq!(found, wanted, "{}", failure_context(&result));
    for diagnostic in &result.diagnostics {
        assert_eq!(diagnostic.rule_name, RULE);
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(diagnostic.message, MESSAGE);
        assert_eq!(diagnostic.help, None);
        assert!(diagnostic.labels.is_empty());
        assert!(diagnostic.fix.is_none());
    }
    assert_eq!(
        (result.error_count, result.warning_count),
        (0, expected.len())
    );
    assert_eq!(result.filename.as_str(), path.to_str().expect("UTF-8 path"));
}
