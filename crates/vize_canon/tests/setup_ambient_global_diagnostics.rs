#[path = "support/script_block_project.rs"]
mod project;

#[test]
fn global_values_and_computed_keys_keep_their_authored_types() {
    let source = "<script setup lang=\"ts\">\ndeclare const list: string | number | { [Symbol.iterator](): Iterator<number> };\ndeclare const doc: typeof document;\ndeclare const settings: typeof Application.settings;\nconst title: string = settings.title;\nconst body: HTMLElement = doc.body;\n</script>\n<template><p v-for=\"item in list\">{{ item }}</p>{{ title }}{{ body }}</template>";
    let globals = "declare namespace Application { const settings: { title: string }; }";
    assert_eq!(
        project::check(&[("src/globals.d.ts", globals), ("src/App.vue", source)]),
        [] as [String; 0]
    );
}

#[test]
fn standard_library_types_remain_available_to_ambient_setup_declarations() {
    let source = "<script setup lang=\"ts\">\nnode.focus();\ndeclare const node: HTMLElement;\ndeclare function load<T extends HTMLElement>(value: T): Promise<T>;\nconst result: Promise<HTMLElement> = load(node);\n</script>\n<template><div>{{ node.tagName }}{{ result }}</div></template>\n";
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        [] as [String; 0]
    );
}

#[test]
fn project_ambient_namespaces_remain_available() {
    let declarations =
        "declare namespace Application { interface Settings { readonly title: string; } }\n";
    let source = "<script setup lang=\"ts\">\ndeclare const settings: Application.Settings;\nconst title: string = settings.title;\n</script>\n<template><div>{{ settings.title }}{{ title }}</div></template>\n";
    assert_eq!(
        project::check(&[("src/globals.d.ts", declarations), ("src/App.vue", source)]),
        [] as [String; 0]
    );
}

#[test]
fn invalid_ambient_types_keep_exact_authored_diagnostics() {
    for (script, expected) in [
        (
            "declare const value: Missing;",
            "src/App.vue(2,22): error TS2304: Cannot find name 'Missing'.",
        ),
        (
            "declare const value: Promise;",
            "src/App.vue(2,22): error TS2314: Generic type 'Promise<T>' requires 1 type argument(s).",
        ),
        (
            "declare const value: HTMLElement;\nvalue.toFixed();",
            "src/App.vue(3,7): error TS2339: Property 'toFixed' does not exist on type 'HTMLElement'.",
        ),
        (
            "declare const value: HTMLElement;\nvalue = document.body;",
            "src/App.vue(3,1): error TS2588: Cannot assign to 'value' because it is a constant.",
        ),
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\n{script}\n</script>\n<template><div /></template>\n"
        );
        assert_eq!(project::check(&[("src/App.vue", &source)]), [expected]);
    }
}

#[test]
fn global_type_errors_do_not_consume_or_detach_suppression_comments() {
    for (script, expected) in [
        (
            "// @ts-expect-error missing type\ndeclare const value: Missing;",
            vec![],
        ),
        (
            "// @ts-expect-error unused\ndeclare const value: HTMLElement;",
            vec!["src/App.vue(2,1): error TS2578: Unused '@ts-expect-error' directive."],
        ),
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\n{script}\n</script>\n<template><div /></template>\n"
        );
        assert_eq!(project::check(&[("src/App.vue", &source)]), expected);
    }
}
