#[path = "support/script_block_project.rs"]
mod project;

#[test]
fn ambient_setup_values_are_typed_without_synthetic_modifier_errors() {
    let source = "<script setup lang=\"ts\">\nlabel.toUpperCase();\ndeclare const label: string;\n</script>\n<template><div>{{ label }}</div></template>\n";
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        [] as [String; 0]
    );
}

#[test]
fn real_ambient_diagnostics_keep_their_authored_positions() {
    for (script, expected) in [
        (
            "declare const label;",
            "src/App.vue(2,15): error TS7005: Variable 'label' implicitly has an 'any' type.",
        ),
        (
            "declare const label: string;\nlabel = 'next';",
            "src/App.vue(3,1): error TS2588: Cannot assign to 'label' because it is a constant.",
        ),
        (
            "declare const label: string;\nlabel.toFixed();",
            "src/App.vue(3,7): error TS2551: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?",
        ),
        (
            "declare const label: string | number;\nlabel.toUpperCase();",
            "src/App.vue(3,7): error TS2339: Property 'toUpperCase' does not exist on type 'string | number'.\nProperty 'toUpperCase' does not exist on type 'number'.",
        ),
        (
            "const prefix = '\u{65e5}\u{672c}\u{8a9e}';\ndeclare const label: string;\nlabel.toFixed();",
            "src/App.vue(4,7): error TS2551: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?",
        ),
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\n{script}\n</script>\n<template><div /></template>\n"
        );
        assert_eq!(
            project::check(&[("src/App.vue", &source)]),
            [expected],
            "{script}"
        );
    }
}

#[test]
fn overloads_and_suppression_directives_remain_effective() {
    for script in [
        "declare function label(value: string): string;\ndeclare function label(value: number): number;\nconst a: string = label('ok');\nconst b: number = label(1);",
        "// @ts-ignore intentional declaration\ndeclare const label: string;",
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\n{script}\n</script>\n<template><div /></template>\n"
        );
        assert_eq!(
            project::check(&[("src/App.vue", &source)]),
            [] as [String; 0],
            "{script}"
        );
    }
}

#[test]
fn invalid_ambient_initializers_keep_the_authored_native_diagnostic() {
    let source = "<script setup lang=\"ts\">\ndeclare let label = 1;\n</script>\n<template><div /></template>\n";
    // Retaining the authored declaration must preserve vue-tsc's TS1039,
    // without an extra OXC report or a synthetic function-scope TS1184.
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        ["src/App.vue(2,21): error TS1039: Initializers are not allowed in ambient contexts."]
    );
}

#[test]
fn template_uses_of_ambient_values_remain_checked() {
    let source = "<script setup lang=\"ts\">\ndeclare const label: string;\n</script>\n<template><div>{{ label.toFixed() }}</div></template>\n";
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        [
            "src/App.vue(4,25): error TS2551: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?"
        ]
    );
}

#[test]
fn unused_expect_error_is_not_silently_consumed_by_synthetic_ts1184() {
    let source = "<script setup lang=\"ts\">\n// @ts-expect-error unused\ndeclare const label: string;\n</script>\n<template><div /></template>\n";
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        ["src/App.vue(2,1): error TS2578: Unused '@ts-expect-error' directive."]
    );
}

#[test]
fn invalid_calls_still_check_every_ambient_overload() {
    let source = "<script setup lang=\"ts\">\ndeclare function label(value: string): string;\ndeclare function label(value: number): number;\nlabel(true);\n</script>\n<template><div /></template>\n";
    assert_eq!(
        project::check(&[("src/App.vue", source)]),
        [
            "src/App.vue(4,7): error TS2769: No overload matches this call.\nThe last overload gave the following error.\nArgument of type 'boolean' is not assignable to parameter of type 'number'."
        ]
    );
}
