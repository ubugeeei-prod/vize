//! Every way a template can use an import as a value, and every look-alike
//! that is not a use. The script uses `Foo` only as a type.

use super::{ALL_TYPES, nothing, report, vue};

fn component(template: &str, rest: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nimport {{ Foo }} from './foo';\ndefineProps<{{ item: Foo }}>();\n</script>\n<template>{template}</template>\n{rest}"
    )
}

#[test]
fn a_template_value_use_keeps_the_import_a_value_import() {
    for template in [
        // Expressions.
        "{{ Foo }}",
        "{{ Foo.label }}",
        "{{ format(Foo) }}",
        "<p :title=\"Foo\" />",
        "<p :Foo />",
        "<p v-bind=\"Foo\" />",
        "<p @click=\"Foo\" />",
        "<p @click=\"Foo()\" />",
        "<p @click=\"(event) => Foo(event)\" />",
        "<input v-model=\"Foo\" />",
        "<p v-if=\"Foo\" />",
        "<p v-if=\"item\" /><p v-else-if=\"Foo\" />",
        "<p v-show=\"Foo\" />",
        "<p v-html=\"Foo\" />",
        "<p v-text=\"Foo\" />",
        "<p v-memo=\"[Foo]\" />",
        "<p v-for=\"entry in Foo\" :key=\"entry\" />",
        "<p v-for=\"entry in [item]\" :key=\"Foo\" />",
        "<p v-custom=\"Foo\" />",
        // Dynamic arguments.
        "<p :[Foo]=\"1\" />",
        "<p @[Foo]=\"item\" />",
        "<p v-custom:[Foo] />",
        "<Bar><template #[Foo]>x</template></Bar>",
        // Component tags, in each casing Vue resolves against setup bindings.
        "<Foo />",
        "<foo />",
        "<Foo.Bar />",
        "<component :is=\"Foo\" />",
        "<div is=\"vue:Foo\" />",
        // A template ref fills the binding of that name.
        "<p ref=\"Foo\" />",
        // A destructuring default of a slot or `v-for` pattern reads the
        // outer binding.
        "<Bar v-slot=\"value = Foo\">{{ value }}</Bar>",
        "<p v-for=\"(value = Foo) in [item]\" :key=\"value\" />",
        "<Bar v-slot=\"{ value = Foo }\">{{ value }}</Bar>",
        "<Bar><template #row=\"{ value = Foo.fallback }\">{{ value }}</template></Bar>",
        "<p v-for=\"{ label = Foo } in [item]\" :key=\"label\">{{ label }}</p>",
        "<Bar v-slot=\"{ [Foo.field]: value }\">{{ value }}</Bar>",
        "<p v-for=\"{ [Foo.field]: value } in [item]\" :key=\"value\" />",
        // An alias in a sibling does not shadow this read.
        "<p v-for=\"Foo in [1]\" :key=\"Foo\" /><p>{{ Foo }}</p>",
    ] {
        assert_eq!(vue(&component(template, "")), nothing(), "{template}");
    }
}

#[test]
fn kebab_tags_and_directives_resolve_camel_case_bindings() {
    for (binding, template) in [
        ("FooBar", "<foo-bar />"),
        ("FooBar", "<FooBar />"),
        ("fooBar", "<foo-bar />"),
        ("vFocus", "<input v-focus />"),
        ("vClickOutside", "<p v-click-outside=\"item\" />"),
    ] {
        let source = format!(
            "<script setup lang=\"ts\">\nimport {{ {binding} }} from './foo';\ndefineProps<{{ item: typeof {binding} }}>();\n</script>\n<template>{template}</template>\n"
        );
        assert_eq!(vue(&source), nothing(), "{binding} {template}");
    }
}

#[test]
fn a_name_the_template_declares_or_never_evaluates_is_not_a_use() {
    for template in [
        "<p />",
        // `v-for` and slot aliases shadow the import.
        "<p v-for=\"Foo in [1]\" :key=\"Foo\">{{ Foo }}</p>",
        "<p v-for=\"(entry, Foo) in [1]\" :key=\"Foo\">{{ Foo }}</p>",
        "<Bar v-slot=\"{ Foo }\">{{ Foo }}</Bar>",
        "<Bar><template #default=\"Foo\">{{ Foo }}</template></Bar>",
        "<p @click=\"(Foo) => Foo\" />",
        // A default that names an enclosing alias, or the pattern's own name.
        "<p v-for=\"Foo in [1]\" :key=\"Foo\"><Bar v-slot=\"{ value = Foo }\">{{ value }}</Bar></p>",
        "<Bar v-slot=\"{ Foo = item }\">{{ Foo }}</Bar>",
        // A static `is` names a registered component, not a binding.
        "<component is=\"Foo\" />",
        // Type positions, text, comments and `v-pre` evaluate nothing.
        "<p :title=\"(item as Foo).label\" />",
        "<p>Foo</p>",
        "<p title=\"Foo\" />",
        "<!-- {{ Foo }} -->",
        "<p v-pre>{{ Foo }}</p>",
        // A member or key spelled like the import is not the import.
        "{{ item.Foo }}",
        "<p :class=\"{ Foo: item }\" />",
        "{{ 'Foo' }}",
    ] {
        assert_eq!(
            vue(&component(template, "")),
            [report(2, ALL_TYPES)],
            "{template}"
        );
    }
}

#[test]
fn a_style_v_bind_is_a_value_use() {
    for style in [
        "<style>p { color: v-bind(Foo); }</style>",
        "<style scoped>p { color: v-bind('Foo.color'); }</style>",
    ] {
        assert_eq!(vue(&component("<p />", style)), nothing(), "{style}");
    }
    assert_eq!(
        vue(&component("<p />", "<style>p { color: red; }</style>")),
        [report(2, ALL_TYPES)]
    );
    // Without a template the style block is still read.
    let script = "<script setup lang=\"ts\">\nimport { Foo } from './foo';\ndefineProps<{ item: typeof Foo }>();\n</script>\n";
    assert_eq!(
        vue(&format!(
            "{script}<style>p {{ color: v-bind(Foo); }}</style>\n"
        )),
        nothing()
    );
    assert_eq!(
        vue(&format!("{script}<style>p {{ color: red; }}</style>\n")),
        [report(2, ALL_TYPES)]
    );
    assert_eq!(vue(script), [report(2, ALL_TYPES)]);
}

#[test]
fn a_block_that_cannot_be_read_rules_no_use_out() {
    for (template, rest) in [
        // An expression that does not parse, and direct `eval`.
        ("{{ broken + }}", ""),
        ("{{ eval('Foo') }}", ""),
        ("<p />", "<style>p { color: v-bind(broken +); }</style>"),
        ("<p />", "<style src=\"./style.css\" />"),
        // Musea variants are templates of their own.
        ("<p />", "<art><variant name=\"a\"><Foo /></variant></art>"),
    ] {
        assert_eq!(
            vue(&component(template, rest)),
            nothing(),
            "{template} {rest}"
        );
    }
    for source in [
        "<script setup lang=\"ts\">\nimport { Foo } from './foo';\ndefineProps<{ item: Foo }>();\n</script>\n<template src=\"./view.html\" />\n",
    ] {
        assert_eq!(vue(source), nothing(), "{source}");
    }
}

#[test]
fn a_pug_template_is_read_through_its_derived_view() {
    let used = "<script setup lang=\"ts\">\nimport { Foo, Bar } from './foo';\ndefineProps<{ item: Foo, other: Bar }>();\n</script>\n<template lang=\"pug\">\np {{ Foo.label }}\n</template>\n";
    assert_eq!(
        vue(used),
        [report(2, "Import \"Bar\" is only used as a type.")]
    );
}
