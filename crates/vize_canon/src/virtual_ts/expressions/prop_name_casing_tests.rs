//! Template attribute names resolve to prop keys the way Vue's `camelize` does.
//!
//! A separator is dropped and the character after it uppercased; nothing else
//! changes. The leading character in particular keeps its case, so a prop
//! declared `Template` stays `Template` instead of being renamed to `template`
//! and reported missing (#3863).

use vize_croquis::{Analyzer, AnalyzerOptions};

use crate::virtual_ts::generate_virtual_ts;
use crate::virtual_ts::helpers::to_camel_case;

fn virtual_ts(script: &str, template: &str) -> std::string::String {
    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    generate_virtual_ts(&summary, Some(script), Some(&root), 0)
        .code
        .as_str()
        .to_string()
}

#[test]
fn camelize_matches_vue_and_never_lowercases_the_leading_character() {
    // A hyphen is dropped and the character after it uppercased.
    assert_eq!(to_camel_case("my-prop"), "myProp");
    assert_eq!(to_camel_case("my-long-prop"), "myLongProp");

    // The leading character keeps its case: Vue matches `:MyProp` to a prop
    // declared `MyProp`, not to one declared `myProp`.
    assert_eq!(to_camel_case("Template"), "Template");
    assert_eq!(to_camel_case("MyProp"), "MyProp");
    assert_eq!(to_camel_case("My-prop"), "MyProp");

    // Vue's `camelizeRE` is `/-(\w)/g`, so an underscore is an ordinary
    // identifier character and a snake_case prop keeps its name.
    assert_eq!(to_camel_case("my_prop"), "my_prop");
    assert_eq!(to_camel_case("_private"), "_private");

    // Already-camel and single-character names pass through.
    assert_eq!(to_camel_case("myProp"), "myProp");
    assert_eq!(to_camel_case("a"), "a");
    assert_eq!(to_camel_case("A"), "A");
    assert_eq!(to_camel_case(""), "");
}

#[test]
fn a_snake_case_prop_keeps_its_name() {
    let code = virtual_ts(
        "import Child from \"./Child.vue\"\nconst value = 1\n",
        r#"<Child :my_prop="value" />"#,
    );

    assert!(
        code.contains(
            "__VizePropValue<__Child_ValueProps_0, 'my_prop', __Child_FallthroughValue_0<'my_prop'>>"
        ),
        "an underscore was treated as a separator:\n{code}"
    );
}

#[test]
fn a_prop_named_template_keeps_its_case_everywhere_it_is_emitted() {
    // The name collides with the SFC `<template>` block, which is what made the
    // lowercasing invisible until a real project bound a prop called `Template`.
    let code = virtual_ts(
        "import Child from \"./Child.vue\"\nconst Template = 'x'\n",
        r#"<Child :Template="Template" :other="1" />"#,
    );

    // Every site that names the prop has to agree with the declared key, or the
    // prop check reports it missing while the parent clearly bound it.
    assert!(
        code.contains(
            "__VizePropValue<__Child_ValueProps_0, 'Template', __Child_FallthroughValue_0<'Template'>>"
        ),
        "prop alias type lost the declared casing:\n{code}"
    );
    assert!(
        code.contains("void __vize_props_nav_0.Template;"),
        "navigation reference lost the declared casing:\n{code}"
    );
    assert!(
        code.contains("\"Template\": Template,"),
        "props object lost the declared casing:\n{code}"
    );
    assert!(
        !code.contains("'template'"),
        "a lowercased prop key survived:\n{code}"
    );
}

#[test]
fn kebab_case_attributes_still_resolve_to_their_camel_case_prop() {
    let code = virtual_ts(
        "import Child from \"./Child.vue\"\nconst value = 1\n",
        r#"<Child :my-prop="value" />"#,
    );

    assert!(
        code.contains(
            "__VizePropValue<__Child_ValueProps_0, 'myProp', __Child_FallthroughValue_0<'my-prop'>>"
        ),
        "kebab attribute did not split prop and fallthrough lookup keys:\n{code}"
    );
}
