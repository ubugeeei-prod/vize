//! Whole-component cases: both script blocks, the `generic` attribute,
//! decorators, and type-only imports a template evaluates.

use super::{ALL_TYPES, nothing, report, reports, vue};

#[test]
fn the_reported_component_reports_only_the_type_import() {
    // `Bar` is read by the template, `Foo` only names a prop type.
    let source = "<script setup lang=\"ts\">\nimport { Bar } from \"./Foo\";\nimport { Foo } from \"./Foo\";\n\nconst { item } = defineProps<{ item: Foo }>();\n</script>\n\n<template>\n  <p>{{ Bar.label }} {{ item.label }}</p>\n</template>\n";
    assert_eq!(vue(source), [report(3, ALL_TYPES)]);
    let result = super::lint(source, "App.vue", None);
    let diagnostic = result.diagnostics.first().unwrap();
    let start = source.find("import { Foo }").unwrap();
    assert_eq!(
        (diagnostic.start as usize, diagnostic.end as usize),
        (start, start + "import { Foo } from \"./Foo\";".len())
    );
}

#[test]
fn both_script_blocks_share_one_module_scope() {
    // The setup block reads the plain block's import, and the reverse.
    let source = "<script lang=\"ts\">\nimport { A, B, C } from './a';\nexport type X = A;\nexport const c = D;\n</script>\n<script setup lang=\"ts\">\nimport { D, E, F } from './d';\nconst b = B;\ntype Y = E | C;\nconsole.log(b);\n</script>\n<template><p>{{ F }}</p></template>\n";
    assert_eq!(
        vue(source),
        [
            report(2, "Imports \"A\" and \"C\" are only used as types."),
            report(7, "Import \"E\" is only used as a type."),
        ]
    );
    // The template sees the plain block's imports through the setup scope.
    let source = "<script lang=\"ts\">\nimport { A } from './a';\nexport type X = A;\n</script>\n<script setup lang=\"ts\">\nconst one = 1;\n</script>\n<template><p>{{ A }} {{ one }}</p></template>\n";
    assert_eq!(vue(source), nothing());
    // A block that does not parse hides its uses.
    let source = "<script lang=\"ts\">\nexport const broken = ;\n</script>\n<script setup lang=\"ts\">\nimport { D } from './d';\ntype Y = D;\n</script>\n";
    assert_eq!(vue(source), nothing());
}

#[test]
fn an_options_component_hands_the_template_only_what_it_returns() {
    // Without `<script setup>` the template cannot reach a module import.
    let source = "<script lang=\"ts\">\nimport { defineComponent } from 'vue';\nimport { Foo } from './foo';\nimport Child from './Child.vue';\nexport default defineComponent({\n  components: { Child },\n  data: () => ({ item: null as Foo | null }),\n});\n</script>\n<template><Child>{{ Foo }}</Child></template>\n";
    assert_eq!(vue(source), [report(3, ALL_TYPES)]);
}

#[test]
fn the_generic_attribute_is_a_type_use() {
    let source = "<script setup lang=\"ts\" generic=\"T extends Base, U = typeof Fallback\">\nimport { Base, Fallback, Other } from './types';\ndefineProps<{ item: T, other: U }>();\n</script>\n<template><p /></template>\n";
    assert_eq!(
        vue(source),
        [report(
            2,
            "Imports \"Base\" and \"Fallback\" are only used as types."
        )]
    );
}

#[test]
fn a_type_only_import_a_template_expression_evaluates_is_reported() {
    let source = "<script setup lang=\"ts\">\nimport type { Shape, Kind } from './shape';\nimport { type Size, Unit } from './size';\nimport type Default from './default';\nconst unit: Kind | Default = Unit;\nconsole.log(unit);\n</script>\n<template><p :title=\"Shape.name\">{{ Size }}</p><component :is=\"Default\" /></template>\n";
    assert_eq!(
        vue(source),
        [
            report(
                2,
                "\"Shape\" is imported as a type, but the template reads it as a value."
            ),
            report(
                3,
                "\"Size\" is imported as a type, but the template reads it as a value."
            ),
            report(
                4,
                "\"Default\" is imported as a type, but the template reads it as a value."
            ),
        ]
    );
}

#[test]
fn a_style_bind_of_a_type_only_import_is_reported() {
    let source = "<script setup lang=\"ts\">\nimport type { Theme } from './theme';\ndefineProps<{ theme: Theme }>();\n</script>\n<template><p /></template>\n<style scoped>\np { color: v-bind('Theme.color'); }\n</style>\n";
    assert_eq!(
        vue(source),
        [report(
            2,
            "\"Theme\" is imported as a type, but a `<style>` `v-bind()` reads it as a value."
        )]
    );
    assert_eq!(
        super::fixed(source, "Component.vue", None),
        source.replace("import type { Theme }", "import { Theme }")
    );
}

#[test]
fn a_template_read_is_left_to_no_undefined_refs_when_that_rule_is_active() {
    const UNDEFINED_REFS: &str = "vue/no-undefined-refs";
    let source = "<script setup lang=\"ts\">\nimport type { Shape } from './shape';\nimport type { Theme } from './theme';\ndefineProps<{ kind: Shape; theme: Theme }>();\n</script>\n<template><p :title=\"Shape.name\" /></template>\n<style scoped>\np { color: v-bind('Theme.color'); }\n</style>\n";
    let both = crate::Linter::with_all()
        .with_enabled_rules(Some(vec![super::RULE.into(), UNDEFINED_REFS.into()]));
    let result = both.lint_sfc(source, "Component.vue");
    let mut found: Vec<(u32, &str, String)> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.start,
                diagnostic.rule_name,
                diagnostic.message.to_string(),
            )
        })
        .collect();
    found.sort();
    let found: Vec<(&str, String)> = found
        .into_iter()
        .map(|(_, rule, message)| (rule, message))
        .collect();
    // One report per read: the template's from the rule that owns template
    // names, the style's from this rule, which nothing else reports.
    assert_eq!(
        found,
        [
            (
                super::RULE,
                "\"Theme\" is imported as a type, but a `<style>` `v-bind()` reads it as a value."
                    .to_string()
            ),
            (
                UNDEFINED_REFS,
                "Variable 'Shape' is not defined".to_string()
            ),
        ]
    );
    assert_eq!(
        vue(source),
        [
            report(
                2,
                "\"Shape\" is imported as a type, but the template reads it as a value."
            ),
            report(
                3,
                "\"Theme\" is imported as a type, but a `<style>` `v-bind()` reads it as a value."
            ),
        ]
    );
}

#[test]
fn a_name_with_another_runtime_source_is_not_reported_as_missing() {
    for source in [
        // A registered or auto-imported component can render the tag, and a
        // directive or a ref name needs no binding either.
        "<script setup lang=\"ts\">\nimport type Card from './Card.vue';\nimport type { vFocus } from './focus';\nconst card = null as InstanceType<typeof Card> | null;\nconsole.log(card, null as typeof vFocus | null);\n</script>\n<template><Card ref=\"Card\" v-focus /></template>\n",
        // The prop of the same name is what the template reads.
        "<script setup lang=\"ts\">\nimport type { User } from './user';\ndefineProps<{ User: User }>();\n</script>\n<template><p>{{ User.name }}</p></template>\n",
        // Props typed by an import this file cannot resolve may declare it.
        "<script setup lang=\"ts\">\nimport type { Props, User } from './missing-module';\ndefineProps<Props>();\nexport type Other = User;\n</script>\n<template><p>{{ User.name }}</p></template>\n",
        // A JavaScript global the template can reach wins over the type name.
        "<script setup lang=\"ts\">\nimport type { Date } from './date';\ndefineProps<{ at: Date }>();\n</script>\n<template><p>{{ Date.now() }}</p></template>\n",
        // A plugin can install a `$`-prefixed property on the component proxy.
        "<script setup lang=\"ts\">\nimport type { $api } from './api';\ndefineProps<{ api: typeof $api }>();\n</script>\n<template><p>{{ $api.version }}</p></template>\n",
        // A template-local alias shadows the import.
        "<script setup lang=\"ts\">\nimport type { User } from './user';\ndefineProps<{ users: User[] }>();\n</script>\n<template><p v-for=\"User in users\" :key=\"User.id\">{{ User.name }}</p></template>\n",
        // A type position in the template is a type use.
        "<script setup lang=\"ts\">\nimport type { User } from './user';\ndefineProps<{ item: unknown }>();\n</script>\n<template><p>{{ (item as User).name }}</p></template>\n",
    ] {
        assert_eq!(vue(source), nothing(), "{source}");
    }
}

#[test]
fn decorator_metadata_positions_are_value_uses() {
    for source in [
        "import Foo from 'foo';\n@deco\nclass A {\n  constructor(foo: Foo) {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  constructor(@deco foo: Foo) {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  foo: Foo;\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  foo(foo: Foo) {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  foo(): Foo {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  foo(@deco foo: Foo) {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  set foo(value: Foo) {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  get foo() {}\n\n  set foo(value: Foo) {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  get foo() {}\n\n  set ['foo'](value: Foo) {}\n}\n",
        "import * as foo from 'foo';\n@deco\nclass A {\n  constructor(foo: foo.Foo) {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  foo: Foo | null;\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  accessor foo: Foo;\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  get foo(): Foo {}\n}\n",
        "import Foo from 'foo';\nclass A {\n  @deco\n  get 0x10() {}\n  set 16(value: Foo) {}\n}\n",
        // A rest parameter serializes its element type.
        "import Foo from 'foo';\nclass A {\n  @deco\n  foo(...rest: Foo[]) {}\n}\n",
        "import Foo from 'foo';\n@deco\nclass A {\n  constructor(...rest: Array<Foo>) {}\n}\n",
    ] {
        assert_eq!(reports(source, "module.ts", None), nothing(), "{source}");
        // Without metadata emit the annotation is an ordinary type use.
        assert_eq!(
            reports(
                source,
                "module.ts",
                Some(serde_json::json!({ "emitDecoratorMetadata": false }))
            ),
            [report(1, ALL_TYPES)],
            "{source}"
        );
    }
}

#[test]
fn only_the_serialized_entity_name_is_a_metadata_value() {
    for source in [
        // No decorator on the member, the class, or a parameter.
        "import Foo from 'foo';\n@deco\nclass A {\n  foo: Foo;\n  bar(foo: Foo): Foo {}\n}\n",
        // Type arguments and element types do not reach the emitted metadata.
        "import Foo from 'foo';\nclass A {\n  @deco\n  foo: Promise<Foo>;\n  @deco\n  bar: Foo[];\n  @deco\n  baz(...rest: Map<string, Foo>) {}\n}\n",
        // An overload signature is not the constructor that is decorated.
        "import Foo from 'foo';\n@deco\nclass A {\n  constructor(foo: Foo);\n  constructor(foo: any) {}\n}\n",
        // A different accessor pair.
        "import Foo from 'foo';\nclass A {\n  @deco\n  get foo() {}\n\n  set bar(value: Foo) {}\n}\n",
        // An undecorated class next to a decorated one.
        "import Foo from 'foo';\n@deco\nclass A {}\nclass B {\n  constructor(foo: Foo) {}\n}\n",
    ] {
        assert_eq!(
            reports(source, "module.ts", None),
            [report(1, ALL_TYPES)],
            "{source}"
        );
    }
}

#[test]
fn a_decorated_class_component_keeps_its_metadata_imports() {
    let source = "<script lang=\"ts\">\nimport { Component, Prop, Vue } from 'vue-facing-decorator';\nimport { User } from './user';\nimport { Role } from './role';\n@Component\nexport default class Profile extends Vue {\n  @Prop() readonly user!: User;\n  role: Role | null = null;\n}\n</script>\n<template><p>{{ user.name }}</p></template>\n";
    assert_eq!(vue(source), [report(4, ALL_TYPES)]);
}
