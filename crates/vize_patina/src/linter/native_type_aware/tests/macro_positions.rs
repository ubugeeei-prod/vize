//! #12880: a script-setup compiler macro must not move the typescript-eslint
//! facts of the macro call or of the code around it.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};
use std::path::{Path, PathBuf};

/// The 39 checks #12879 added, as the #12880 report enables them.
const RULES: &[&str] = &[
    "type/await-thenable",
    "type/no-array-delete",
    "type/no-base-to-string",
    "type/no-confusing-void-expression",
    "type/no-deprecated",
    "type/no-duplicate-type-constituents",
    "type/no-for-in-array",
    "type/no-generated-empty-object-type",
    "type/no-implied-eval",
    "type/no-meaningless-void-operator",
    "type/no-misused-promises",
    "type/no-misused-spread",
    "type/no-mixed-enums",
    "type/no-redundant-type-constituents",
    "type/no-unnecessary-boolean-literal-compare",
    "type/no-unnecessary-condition",
    "type/no-unnecessary-template-expression",
    "type/no-unnecessary-type-arguments",
    "type/no-unnecessary-type-assertion",
    "type/no-unnecessary-type-conversion",
    "type/no-unsafe-argument",
    "type/no-unsafe-assignment",
    "type/no-unsafe-call",
    "type/no-unsafe-enum-assignment",
    "type/no-unsafe-enum-comparison",
    "type/no-unsafe-member-access",
    "type/no-unsafe-return",
    "type/no-unsafe-unary-minus",
    "type/only-throw-error",
    "type/prefer-promise-reject-errors",
    "type/prefer-reduce-type-parameter",
    "type/prefer-return-this-type",
    "type/related-getter-setter-pairs",
    "type/require-await",
    "type/restrict-plus-operands",
    "type/restrict-template-expressions",
    "type/return-await",
    "type/unbound-method",
    "type/use-unknown-in-catch-callback-variable",
];

/// The reporter's component with one macro, pinned for the differential run.
const MODEL_ONLY: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/patina-compat-type-aware-macro-positions-12880/ModelOnly.vue.txt"
);

/// The reporter's control component without a macro.
const NO_MACROS: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/patina-compat-type-aware-macro-positions-12880/NoMacros.vue.txt"
);

/// Each macro, then a statement that reads what the macro returned.
const MACROS: &[(&str, &str)] = &[
    (
        "defineModel",
        "const model = defineModel<boolean>({ default: false });\nmodel.value = !model.value;",
    ),
    (
        "named defineModel",
        "const open = defineModel<boolean>(\"open\", { required: true });\nconst openText = `open: ${String(open.value)}`;",
    ),
    (
        "defineProps",
        "const props = defineProps<{ title: string; count?: number }>();\nconst heading = `${props.title}!`;",
    ),
    (
        "runtime defineProps",
        "const props = defineProps({ title: { type: String, required: true } });\nconst heading = `${props.title}!`;",
    ),
    (
        "withDefaults",
        "const props = withDefaults(defineProps<{ title?: string }>(), { title: \"x\" });\nconst heading = `${props.title}!`;",
    ),
    (
        "destructured props",
        "const { title = \"x\", count = 0 } = defineProps<{ title?: string; count?: number }>();\nconst heading = `${title}: ${count + 1}`;",
    ),
    (
        "defineEmits",
        "const emit = defineEmits<{ change: [value: number] }>();\nfunction notify(): void {\n  emit(\"change\", 1);\n}",
    ),
    (
        "defineSlots",
        "const slots = defineSlots<{ default(props: { item: string }): unknown }>();\nconst hasDefault = typeof slots.default === \"function\";",
    ),
    (
        "defineExpose",
        "function focus(): void {}\ndefineExpose({ focus });",
    ),
    (
        "defineOptions",
        "defineOptions({ name: \"MacroShapes\", inheritAttrs: false });",
    ),
];

const TEMPLATE_STRING_COLUMN: u32 = 29;

/// The reporter's tsconfig.
const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "jsx": "preserve",
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
"#;

/// The reporter's project layout with a Vue from the workspace's pnpm store,
/// so `ref` and `defineModel` have their real types. A checkout without an
/// install skips, unless `VIZE_TEST_REQUIRE_TSGO` makes that fatal.
fn vue_project() -> Option<tempfile::TempDir> {
    if !corsa_available() {
        return None;
    }
    let Some(vue) = workspace_vue() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "the #12880 project needs Vue 3.5 or later; run `pnpm install`"
        );
        return None;
    };
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("macro-positions-12880-")
        .tempdir_in(workspace)
        .expect("reported project");
    let root = project.path();
    std::fs::create_dir_all(root.join("src")).expect("src");
    std::fs::create_dir_all(root.join("node_modules")).expect("node_modules");
    std::fs::write(root.join("tsconfig.json"), TSCONFIG).expect("tsconfig");
    std::fs::write(root.join("package.json"), "{ \"private\": true }\n").expect("package");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&vue, root.join("node_modules/vue")).expect("Vue link");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&vue, root.join("node_modules/vue")).expect("Vue link");
    Some(project)
}

/// The newest Vue 3.5+ in the workspace's pnpm store, if installed.
fn workspace_vue() -> Option<PathBuf> {
    let store = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../node_modules/.pnpm");
    let mut found = std::fs::read_dir(store)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.strip_prefix("vue@3.")
                .and_then(|rest| rest.split(['.', '-', '_']).next())
                .and_then(|minor| minor.parse::<u32>().ok())
                .is_some_and(|minor| minor >= 5)
        })
        .map(|entry| entry.path().join("node_modules/vue"))
        .filter(|vue| vue.join("package.json").is_file())
        .collect::<Vec<_>>();
    found.sort();
    found.pop()
}

fn lint_in(
    project: &Path,
    linter: &Linter,
    source: &str,
    name: &str,
) -> Vec<(u32, u32, &'static str)> {
    let path = project.join("src").join(name);
    std::fs::write(&path, source).expect("component");
    typescript_eslint_reports(linter, source, path.to_str().expect("UTF-8 path"))
}

#[test]
fn issue_reproduction_reads_no_wrong_types() {
    let Some(project) = vue_project() else {
        return;
    };
    let linter = Linter::with_preset(LintPreset::Opinionated);
    for (name, source) in [("ModelOnly.vue", MODEL_ONLY), ("NoMacros.vue", NO_MACROS)] {
        let reports = lint_in(project.path(), &linter, source, name);
        assert_eq!(reports, Vec::<(u32, u32, &str)>::new(), "{name}");
    }
}

/// An array in a template string is a finding before and after the macro;
/// a boolean ref read through `.value` is clean on both sides.
#[test]
fn each_macro_keeps_facts_before_and_after_it() {
    let Some(project) = vue_project() else {
        return;
    };
    let linter = Linter::with_preset(LintPreset::Opinionated);
    for (name, macro_lines) in MACROS {
        let source = macro_source(macro_lines);
        let macro_line_count = macro_lines.lines().count() as u32;
        let before_line = 5;
        let after_line = before_line + 3 + macro_line_count;
        let reports = lint_in(project.path(), &linter, &source, "MacroShapes.vue");
        assert_eq!(
            reports,
            vec![
                (
                    before_line,
                    TEMPLATE_STRING_COLUMN,
                    "type/restrict-template-expressions"
                ),
                (
                    after_line,
                    TEMPLATE_STRING_COLUMN,
                    "type/restrict-template-expressions"
                ),
            ],
            "{name}:\n{source}"
        );
    }
}

fn macro_source(macro_lines: &str) -> std::string::String {
    let mut source = std::string::String::from(
        "<script setup lang=\"ts\">\nimport { ref } from \"vue\";\n\nconst numbers = [1, 2, 3];\nconst beforeList = `list: ${numbers}`;\nconst beforeFlag = ref(false);\nconst beforeLabel = `open: ${beforeFlag.value}`;\n",
    );
    source.push_str(macro_lines);
    source.push_str(
        "\nconst afterList_ = `list: ${numbers}`;\nconst afterFlag = ref(false);\nconst afterLabel = `open: ${afterFlag.value}`;\nfunction toggle(): void {\n  afterFlag.value = !afterFlag.value;\n}\n</script>\n\n<template>\n  <button type=\"button\" @click=\"toggle\">{{ beforeList }}{{ beforeLabel }}{{ afterList_ }}{{ afterLabel }}</button>\n</template>\n",
    );
    source
}

fn typescript_eslint_reports(
    linter: &Linter,
    source: &str,
    filename: &str,
) -> Vec<(u32, u32, &'static str)> {
    let result = lint_sfc_with_corsa(linter, source, filename);
    let mut reports = result
        .diagnostics
        .iter()
        .filter(|diagnostic| RULES.contains(&diagnostic.rule_name))
        .map(|diagnostic| {
            let (line, column) = line_column(source, diagnostic.start);
            (line, column, diagnostic.rule_name)
        })
        .collect::<Vec<_>>();
    reports.sort_unstable();
    reports
}

fn line_column(source: &str, offset: u32) -> (u32, u32) {
    let mut line = 1u32;
    let mut column = 1u32;
    for (index, character) in source.char_indices() {
        if index >= offset as usize {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
