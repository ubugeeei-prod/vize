#[test]
fn a_destructured_const_rewrites_the_var_prefix() {
    let script = "const { errors, meta } = useForm()\nlet attempts = 0\n";
    let hover = "```typescript\nvar errors: Ref<string[]>\n```";
    assert_eq!(
        super::align_leading_var(hover, script, Some("ts"), "errors").as_deref(),
        Some("```typescript\nconst errors: Ref<string[]>\n```")
    );
    let hover = "```typescript\nvar attempts: number\n```";
    assert_eq!(
        super::align_leading_var(hover, script, Some("ts"), "attempts").as_deref(),
        Some("```typescript\nlet attempts: number\n```")
    );
}

#[test]
fn unknown_names_prefix_words_and_non_var_hovers_stay_untouched() {
    let script = "const counter = 1\n";
    // Hover text names a longer identifier than the hovered word.
    let hover = "```typescript\nvar counter: number\n```";
    assert_eq!(
        super::align_leading_var(hover, script, Some("ts"), "count"),
        None
    );
    // The word has no top-level declaration.
    assert_eq!(
        super::align_leading_var(hover, script, Some("ts"), "missing"),
        None
    );
    // The quick info does not open with `var`.
    let hover = "```typescript\nconst counter: 1\n```";
    assert_eq!(
        super::align_leading_var(hover, script, Some("ts"), "counter"),
        None
    );
}

#[test]
fn a_jsx_initializer_resolves_under_tsx_and_jsx_langs() {
    // `<script setup lang="tsx">`: parsed as TypeScript, `<Badge />` reads as
    // a type assertion and the declaration never resolves.
    let script = "const badge = <Badge count={1} />\n";
    let hover = "```typescript\nvar badge: JSX.Element\n```";
    assert_eq!(
        super::align_leading_var(hover, script, Some("tsx"), "badge").as_deref(),
        Some("```typescript\nconst badge: JSX.Element\n```")
    );
    assert_eq!(
        super::align_leading_var(hover, script, Some("jsx"), "badge").as_deref(),
        Some("```typescript\nconst badge: JSX.Element\n```")
    );
    // A JSX statement ahead of the hovered `let` must not derail the parse.
    let script = "const badge = <Badge count={1} />\nlet attempts = 0\n";
    let hover = "```typescript\nvar attempts: number\n```";
    assert_eq!(
        super::align_leading_var(hover, script, Some("tsx"), "attempts").as_deref(),
        Some("```typescript\nlet attempts: number\n```")
    );
}

/// The offset of `needle`'s nth occurrence, as a template-relative
/// hover position.
fn offset_of(template: &str, needle: &str, nth: usize) -> u32 {
    template.match_indices(needle).nth(nth).unwrap().0 as u32
}

#[test]
fn a_v_for_alias_parameter_rewrites_to_const() {
    let template = "<ul><li v-for=\"(it, index) in users\" :key=\"it.id\">{{ it.name }}</li></ul>";
    // The alias at a use site and at its own declaration.
    let hover = "```typescript\n(parameter) it: User\n```";
    assert_eq!(
        super::align_v_for_parameter(hover, template, "it", offset_of(template, "it.name", 0))
            .as_deref(),
        Some("```typescript\nconst it: User\n```")
    );
    let hover = "```typescript\n(parameter) index: number\n```";
    assert_eq!(
        super::align_v_for_parameter(hover, template, "index", offset_of(template, "index", 0))
            .as_deref(),
        Some("```typescript\nconst index: number\n```")
    );
    // A non-alias parameter (an event handler's) keeps the checker's answer.
    let hover = "```typescript\n(parameter) event: MouseEvent\n```";
    assert_eq!(
        super::align_v_for_parameter(hover, template, "event", offset_of(template, "users", 0)),
        None
    );
}

#[test]
fn a_parameter_named_like_an_alias_keeps_the_checker_answer() {
    // The handler parameter shadows the alias inside the loop element, and
    // names it again outside the loop entirely.
    let template = concat!(
        "<ul><li v-for=\"item in items\" @click=\"(item) => item.id\">{{ item.name }}</li></ul>",
        "<button @click=\"(item) => item.id\">x</button>"
    );
    let hover = "```typescript\n(parameter) item: MouseEvent\n```";
    // Shadowing parameter inside the v-for element: the handler's binding wins.
    assert_eq!(
        super::align_v_for_parameter(hover, template, "item", offset_of(template, "item.id", 0)),
        None
    );
    // Same name in a handler outside the loop: the alias is not in effect.
    assert_eq!(
        super::align_v_for_parameter(hover, template, "item", offset_of(template, "item.id", 1)),
        None
    );
    // The alias itself still resolves inside the loop body.
    let hover = "```typescript\n(parameter) item: Item\n```";
    assert_eq!(
        super::align_v_for_parameter(hover, template, "item", offset_of(template, "item.name", 0))
            .as_deref(),
        Some("```typescript\nconst item: Item\n```")
    );
}

#[test]
fn an_imported_component_presents_its_import_not_the_machinery() {
    let script = "import Child from \"./Child.vue\";\nimport { Widget as W } from \"./kit\";\nconst local = 1;\nvoid local;\n";
    assert_eq!(
        super::imported_component_quick_info(script, None, "Child").as_deref(),
        Some("```typescript\nimport Child\n```")
    );
    // Aliased named imports bind their local name.
    assert_eq!(
        super::imported_component_quick_info(script, None, "W").as_deref(),
        Some("```typescript\nimport W\n```")
    );
    // A non-imported name keeps the checker's answer.
    assert_eq!(
        super::imported_component_quick_info(script, None, "local"),
        None
    );
    assert_eq!(
        super::imported_component_quick_info(script, None, "Missing"),
        None
    );
}

#[test]
fn a_type_only_import_keeps_the_checker_answer() {
    // A declaration-level `import type` binds a type, not a component.
    let script = "import type Child from \"./Child.vue\";\n";
    assert_eq!(
        super::imported_component_quick_info(script, Some("ts"), "Child"),
        None
    );
    // Same for a specifier-level `type` qualifier, while a runtime specifier
    // in the very same declaration still resolves.
    let script = "import { type Child, Widget } from \"./kit\";\n";
    assert_eq!(
        super::imported_component_quick_info(script, Some("ts"), "Child"),
        None
    );
    assert_eq!(
        super::imported_component_quick_info(script, Some("ts"), "Widget").as_deref(),
        Some("```typescript\nimport Widget\n```")
    );
}
