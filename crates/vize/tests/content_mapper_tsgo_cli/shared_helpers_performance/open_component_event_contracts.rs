use super::*;

#[test]
fn standard_tsgo_keeps_undeclared_component_events_open() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping open component listener receipt: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let receipt = tempfile::Builder::new()
        .prefix("content-mapper-open-component-events-")
        .tempdir_in(&cases_root)
        .unwrap();
    let app = include_str!(
        "../../../../../tests/_fixtures/differential/typechecker/component-event-tuples/unresolved-valid/src/App.vue.txt"
    );
    let child = include_str!(
        "../../../../../tests/_fixtures/differential/typechecker/component-event-tuples/unresolved-valid/src/UnknownChild.vue.txt"
    );
    for shared_helpers in [false, true] {
        for (owner, macros, fallthrough, errors) in [
            ("undeclared", "", None, 0),
            ("undeclared-explicit-true", "", Some(true), 1),
            (
                "unrelated-declared",
                "defineEmits<{ change: [] }>();",
                None,
                1,
            ),
            ("empty-declared", "defineEmits<{}>();", None, 1),
            ("empty-runtime", "defineEmits([]);", None, 1),
            ("runtime-unrelated", "defineEmits([\"change\"]);", None, 1),
            (
                "options-array-unrelated",
                "export default { emits: [\"change\"] }",
                None,
                1,
            ),
            (
                "options-empty-array",
                "export default { emits: [] }",
                None,
                0,
            ),
            (
                "options-object-unrelated",
                "export default { emits: { change: () => true } }",
                None,
                1,
            ),
            (
                "owned-emits",
                "defineEmits<{ select: [key: string, path: string[]] }>();",
                None,
                0,
            ),
            (
                "owned-prop",
                "defineProps<{ onSelect?: (key: string, path: string[]) => void }>();",
                None,
                0,
            ),
            (
                "owned-emits-wrong",
                "defineEmits<{ select: [key: string, path: string[]] }>();",
                None,
                1,
            ),
            (
                "owned-prop-wrong",
                "defineProps<{ onSelect?: (key: string, path: string[]) => void }>();",
                None,
                1,
            ),
            ("generic-undeclared", "", None, 0),
            (
                "generic-unrelated",
                "defineEmits<{ change: [] }>();",
                None,
                1,
            ),
            ("authored-helper-names", "", None, 0),
            ("native-element", "", None, 1),
        ] {
            let mode = if shared_helpers { "shared" } else { "embedded" };
            let project = receipt.path().join(cstr!("{mode}-{owner}").as_str());
            prepare_mapper_project(&project, shared_helpers);
            let source_root = project.join("src");
            std::fs::create_dir_all(&source_root).unwrap();
            let mut child_source = child.replace("</script>", &cstr!("{macros}</script>"));
            if owner.starts_with("options-") {
                child_source = child_source.replace("<script setup", "<script");
            }
            if owner.starts_with("generic-") {
                child_source = child_source.replace("lang=\"ts\"", "lang=\"ts\" generic=\"T\"");
            }
            let mut app_source = app.to_owned();
            if owner.ends_with("-wrong") {
                app_source = app_source.replace("path: string[]", "path: number[]");
            }
            if owner == "authored-helper-names" {
                app_source = app_source.replace(
                    "</script>",
                    "type $LiteralKeys<T> = never; type PropertyKey = never;\n</script>",
                );
            }
            if owner == "native-element" {
                app_source = app_source.replace("<UnknownChild @select", "<div @select");
            }
            std::fs::write(source_root.join("App.vue"), app_source).unwrap();
            std::fs::write(source_root.join("UnknownChild.vue"), child_source).unwrap();
            let mut config = json!({
                "compilerOptions": {
                    "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler",
                    "strict": true, "noEmit": true, "skipLibCheck": true,
                },
                "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
                "include": ["src"],
            });
            if let Some(enabled) = fallthrough {
                config["vueCompilerOptions"] = json!({ "fallthroughAttributes": enabled });
            }
            std::fs::write(
                project.join("tsconfig.json"),
                serde_json::to_vec_pretty(&config).unwrap(),
            )
            .unwrap();
            let output = check_project(&tsgo, &project, "tsconfig.json");
            assert_eq!(
                output.stderr,
                b"",
                "{mode}-{owner}: {}",
                output_text(&output)
            );
            let stdout = std::str::from_utf8(&output.stdout).unwrap();
            let expected = if errors == 0 {
                String::new()
            } else if owner == "native-element" {
                String::from(
                    "src/App.vue(8,25): error TS2345: Argument of type '(key: string, path: string[]) => void' is not assignable to parameter of type '(_e: Event) => unknown'.\n  Target signature provides too few arguments. Expected 2 or more, but got 1.\n",
                )
            } else if owner.ends_with("-wrong") {
                let result = if owner == "owned-prop-wrong" {
                    "void"
                } else {
                    "any"
                };
                cstr!(
                    "src/App.vue(8,26): error TS2322: Type '(key: string, path: number[]) => void' is not assignable to type '(key: string, path: string[]) => {result}'.\n  Types of parameters 'path' and 'path' are incompatible.\n    Type 'string[]' is not assignable to type 'number[]'.\n      Type 'string' is not assignable to type 'number'.\n"
                ).to_string()
            } else {
                String::from(
                    "src/App.vue(8,26): error TS2322: Type '(key: string, path: string[]) => void' is not assignable to type '(payload: Event) => void'.\n  Target signature provides too few arguments. Expected 2 or more, but got 1.\n",
                )
            };
            assert_eq!(stdout, expected, "{mode}-{owner}: {}", output_text(&output));
            assert_eq!(
                output.status.code(),
                Some(if errors == 0 { 0 } else { 2 }),
                "{mode}-{owner}: {}",
                output_text(&output)
            );
        }
    }
}
