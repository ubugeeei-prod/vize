use vize_canon::VirtualProject;

fn virtual_source(source: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("App.vue");
    std::fs::write(&path, source).unwrap();
    let mut project = VirtualProject::new(dir.path()).unwrap();
    project.register_vue_file(&path, source).unwrap();
    project.find_by_original(&path).unwrap().content.to_string()
}

fn setup(script: &str) -> String {
    format!("<script setup lang=\"ts\">\n{script}\n</script>\n<template><div /></template>")
}

fn authored_setup_body(output: &str) -> &str {
    output
        .split_once("  // User setup code\n")
        .unwrap()
        .1
        .split_once("  // @vize-map:")
        .unwrap()
        .0
        .trim()
}

fn assert_module_declaration(source: &str, declaration: &str, hoisted: bool) {
    let output = virtual_source(source);
    let setup = output.find("// ========== Setup Scope ==========").unwrap();
    let declaration_at = output
        .find(declaration)
        .unwrap_or_else(|| panic!("{output}"));
    assert_eq!(declaration_at < setup, hoisted, "{output}");
    assert_eq!(output.matches(declaration).count(), 1, "{output}");
}

#[test]
fn independent_ambient_values_reach_module_scope_verbatim() {
    for declaration in [
        "declare const label: string;",
        "declare let label: string | number;",
        "declare var label: string;",
        "declare function label<T>(value: T): T;",
        "declare const label = 'fixed';",
    ] {
        assert_module_declaration(&setup(declaration), declaration, true);
    }
}

#[test]
fn ambient_global_type_references_retain_module_visibility() {
    for declaration in [
        "declare const label: HTMLElement;",
        "declare const label: ReadonlyArray<Promise<Date>>;",
        "declare function label<T extends HTMLElement>(value: T): Promise<T>;",
        "declare const label: Application.Settings;",
        "declare const label: Missing;",
        "declare class Label { value: HTMLElement; }",
    ] {
        assert_module_declaration(&setup(declaration), declaration, true);
    }
}

#[test]
fn hoisted_type_and_ambient_dependencies_remain_visible() {
    let declarations = [
        "declare const first: Label;",
        "declare const second: typeof first;",
    ];
    let source = setup(&format!(
        "type Label = string;\n{}",
        declarations.join("\n")
    ));
    for declaration in declarations {
        assert_module_declaration(&source, declaration, true);
    }
}

#[test]
fn setup_local_dependencies_are_not_lifted_or_widened() {
    let source = setup(
        "const local = 'fixed';\ndeclare const first: typeof local;\ndeclare const second: typeof first;",
    );
    for declaration in [
        "declare const first: typeof local;",
        "declare const second: typeof first;",
    ] {
        assert_module_declaration(&source, declaration, false);
    }
}

#[test]
fn global_type_wrappers_do_not_hide_setup_local_dependencies() {
    let script = "class Local {}\ndeclare const first: Local;\ndeclare const second: ReadonlyArray<typeof first>;";
    assert_eq!(
        authored_setup_body(&virtual_source(&setup(script))),
        script.replace('\n', "\n  ")
    );
}

#[test]
fn generic_and_classic_script_scopes_remain_separate() {
    let declaration = "declare const label: string;";
    let generic = setup(declaration).replace("lang=\"ts\"", "lang=\"ts\" generic=\"T\"");
    assert_module_declaration(&generic, declaration, false);
    let classic = format!(
        "<script lang=\"ts\">const label = 1;</script>\n{}",
        setup(declaration)
    );
    assert_module_declaration(&classic, declaration, false);
}

#[test]
fn nested_declarations_keep_their_authored_scope() {
    let declaration = "declare const label: string;";
    assert_module_declaration(
        &setup(&format!("function run() {{ {declaration} }}")),
        declaration,
        false,
    );
}

#[test]
fn adjacent_code_is_never_lost_by_line_based_emission() {
    let declaration = "declare const label: string;";
    let source = setup(&format!("{declaration} const invalid: number = 'bad';"));
    assert_module_declaration(&source, declaration, true);
    assert_eq!(
        authored_setup_body(&virtual_source(&source)),
        "const invalid: number = 'bad';"
    );
}

#[test]
fn adjacent_local_captures_and_comment_scopes_stay_in_setup() {
    for script in [
        "const local = 'fixed'; declare const label: typeof local;",
        "// @ts-expect-error assignment\nconst invalid: number = 'bad'; declare const label: string;",
        "// @ts-ignore assignment\ndeclare const label: string; const invalid: number = 'bad';",
        "const before = 1; /** documented label */ declare const label: string;",
        "const before = 1; declare const label: {\n// @ts-ignore assignment\ntext: string; }; const invalid: number = 'bad';",
    ] {
        let output = virtual_source(&setup(script));
        assert_eq!(authored_setup_body(&output), script.replace('\n', "\n  "));
    }
}

#[test]
fn runtime_redeclarations_are_not_split_between_scopes() {
    let declaration = "declare var label: string;";
    assert_module_declaration(
        &setup(&format!("{declaration}\nvar label = 'fixed';")),
        declaration,
        false,
    );
}

#[test]
fn duplicate_bindings_and_unresolved_setup_helpers_fail_closed() {
    for script in [
        "declare const label: string;\nconst label = 1;",
        "declare const label: typeof defineProps;",
        "declare const label: defineProps;",
        "declare const label: typeof document;",
    ] {
        let declaration = script.lines().next().unwrap();
        assert_module_declaration(&setup(script), declaration, false);
    }
}

#[test]
fn overloaded_signatures_move_together() {
    let declarations = [
        "declare function label(value: string): string;",
        "declare function label(value: number): number;",
    ];
    let source = setup(&declarations.join("\n"));
    for declaration in declarations {
        assert_module_declaration(&source, declaration, true);
    }
}

#[test]
fn multiline_declarations_keep_their_full_authored_text() {
    let declaration = "declare const label: {\r\n  text: string;\r\n};";
    assert_module_declaration(&setup(declaration), declaration, true);
}

#[test]
fn suppression_comments_follow_the_declaration() {
    let declaration = "// @ts-ignore intentional declaration\ndeclare const label: string;";
    assert_module_declaration(&setup(declaration), declaration, true);
}

#[test]
fn long_dependency_chains_keep_all_transitive_local_captures_in_setup() {
    let mut script = String::from("const local = 'fixed';\n");
    for index in (0..256).rev() {
        let dependency = if index == 0 {
            "local".to_string()
        } else {
            format!("value_{}", index - 1)
        };
        script.push_str(&format!(
            "declare const value_{index}: typeof {dependency};\n"
        ));
    }
    let output = virtual_source(&setup(&script));
    assert_eq!(
        authored_setup_body(&output),
        script.trim().replace('\n', "\n  ")
    );
    assert_eq!(output.matches("declare const value_").count(), 256);
}

#[test]
fn large_overload_sets_are_kept_as_one_declaration_group() {
    let script: String = (0..256)
        .map(|index| format!("declare function label(value: {index}): {index};\n"))
        .collect();
    let output = virtual_source(&setup(&script));
    let setup = output.find("// ========== Setup Scope ==========").unwrap();
    assert_eq!(
        output[..setup].matches("declare function label(").count(),
        256
    );
    assert_eq!(authored_setup_body(&output), "");
}

#[test]
fn one_local_capture_or_runtime_implementation_blocks_the_whole_overload_group() {
    for script in [
        "const local = 1;\ndeclare function label(value: string): string;\ndeclare function label(value: typeof local): number;\ndeclare const alias: typeof label;",
        "declare function label(value: string): string;\nfunction label(value: string) { return value; }",
    ] {
        let output = virtual_source(&setup(script));
        assert_eq!(authored_setup_body(&output), script.replace('\n', "\n  "));
    }
}
