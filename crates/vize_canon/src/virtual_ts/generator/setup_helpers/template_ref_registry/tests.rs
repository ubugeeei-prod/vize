use vize_croquis::Croquis;
use vize_relief::BindingType;
use vize_s0::{CompactString, FxHashSet};

use crate::virtual_ts::types::VirtualTsOptions;

fn registry_of(template: &str) -> Option<vize_s0::String> {
    registry_for("const box = useTemplateRef('box')", template).map(|registry| registry.body)
}

fn registry_for(script: &str, template: &str) -> Option<super::TemplateRefRegistry> {
    registry_with_summary(script, template, &Croquis::default())
}

fn registry_with_summary(
    script: &str,
    template: &str,
    summary: &Croquis,
) -> Option<super::TemplateRefRegistry> {
    registry_with_summary_and_options(script, template, summary, &VirtualTsOptions::default())
}

fn registry_with_summary_and_options(
    script: &str,
    template: &str,
    summary: &Croquis,
    options: &VirtualTsOptions,
) -> Option<super::TemplateRefRegistry> {
    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    super::template_ref_registry(
        summary,
        options,
        Some(script),
        Some(&root),
        &FxHashSet::<CompactString>::default(),
    )
}

#[test]
fn a_script_that_never_names_use_template_ref_registers_nothing() {
    assert!(registry_for("const label = 'hi'", r#"<div ref="box" />"#).is_none());
}

#[test]
fn static_plain_element_refs_register_their_tag() {
    assert_eq!(
        registry_of(r#"<div ref="box" /><svg ref="pic" />"#).as_deref(),
        Some(r#" "box": __VizeDomElement<"div">; "pic": __VizeDomElement<"svg", true>; "#)
    );
}

#[test]
fn svg_descendants_resolve_through_the_svg_tag_map() {
    assert_eq!(
        registry_of(r#"<a ref="html" /><svg><a ref="vector" /></svg>"#).as_deref(),
        Some(r#" "html": __VizeDomElement<"a">; "vector": __VizeDomElement<"a", true>; "#)
    );
    assert_eq!(
        registry_of(r#"<svg><foreignObject><a ref="escaped" /></foreignObject></svg>"#).as_deref(),
        Some(r#" "escaped": __VizeDomElement<"a">; "#)
    );
    assert_eq!(
        registry_of(r#"<circle ref="shape" />"#).as_deref(),
        Some(r#" "shape": __VizeDomElement<"circle", true>; "#)
    );
}

#[test]
fn ref_names_are_escaped_as_typescript_string_literals() {
    assert_eq!(
        registry_of(r#"<div ref="path\name" />"#).as_deref(),
        Some(r#" "path\\name": __VizeDomElement<"div">; "#)
    );
}

#[test]
fn unpinnable_refs_stay_out_of_the_registry() {
    assert_eq!(registry_of(r#"<div :ref="target" />"#), None);
    assert_eq!(
        registry_of(r#"<li v-for="it in xs" :key="it" ref="rows" />"#),
        None
    );
    assert_eq!(
        registry_of(r#"<Child v-for="it in xs" :key="it.id" ref="child" />"#),
        None
    );
    assert_eq!(registry_of(r#"<div ref="dup" /><span ref="dup" />"#), None);
}

#[test]
fn component_refs_register_the_component_public_instance() {
    let mut summary = Croquis::default();
    summary.bindings.add("Child", BindingType::SetupConst);
    assert_eq!(
        registry_with_summary(
            "import { useTemplateRef } from 'vue';\nimport Child from './Child.vue';\nconst child = useTemplateRef('child')",
            r#"<Child ref="child" />"#,
            &summary,
        )
        .map(|registry| registry.body)
        .as_deref(),
        Some(r#" "child": __VizeTemplateComponentRef<typeof Child>; "#)
    );
}

#[test]
fn native_element_refs_do_not_become_component_refs_from_same_named_setup_bindings() {
    let mut summary = Croquis::default();
    summary.bindings.add("canvas", BindingType::SetupConst);
    summary.bindings.add("img", BindingType::SetupConst);
    assert_eq!(
        registry_with_summary(
            "import { useTemplateRef } from 'vue';\nconst canvas = useTemplateRef('canvas');\nconst img = useTemplateRef('img');",
            r#"<canvas ref="canvas" /><img ref="img" />"#,
            &summary,
        )
        .map(|registry| registry.body)
        .as_deref(),
        Some(r#" "canvas": __VizeDomElement<"canvas">; "img": __VizeDomElement<"img">; "#)
    );
}

#[test]
fn unknown_foreign_namespace_elements_do_not_become_component_refs_from_setup_bindings() {
    let mut summary = Croquis::default();
    summary.bindings.add("shape", BindingType::SetupConst);
    summary.bindings.add("glyph", BindingType::SetupConst);
    assert_eq!(
        registry_with_summary(
            "import { useTemplateRef } from 'vue';\nconst shape = useTemplateRef('shape');\nconst glyph = useTemplateRef('glyph');",
            r#"<svg><shape ref="shape" /></svg><math><glyph ref="glyph" /></math>"#,
            &summary,
        )
        .map(|registry| registry.body)
        .as_deref(),
        Some(r#" "shape": __VizeDomElement<"shape", true>; "glyph": __VizeDomElement<"glyph">; "#)
    );
}

#[test]
fn component_refs_record_component_helper_without_guessing_from_registry_text() {
    let mut summary = Croquis::default();
    summary.bindings.add("Child", BindingType::SetupConst);
    let registry = registry_with_summary(
        "import { useTemplateRef } from 'vue';\nimport Child from './Child.vue';\nconst tricky = useTemplateRef('__VizeDomElement')",
        r#"<Child ref="__VizeDomElement" />"#,
        &summary,
    )
    .expect("component ref registry");

    assert_eq!(
        registry.body.as_str(),
        r#" "__VizeDomElement": __VizeTemplateComponentRef<typeof Child>; "#
    );
    assert!(!registry.includes_dom_element);
    assert!(registry.includes_component);
}

#[test]
fn kebab_case_component_refs_use_the_declared_component_binding() {
    let mut summary = Croquis::default();
    summary.bindings.add("MyWidget", BindingType::SetupConst);
    assert_eq!(
        registry_with_summary(
            "import { useTemplateRef } from 'vue';\nimport MyWidget from './MyWidget.vue';\nconst widget = useTemplateRef('widget')",
            r#"<my-widget ref="widget" />"#,
            &summary,
        )
        .map(|registry| registry.body)
        .as_deref(),
        Some(r#" "widget": __VizeTemplateComponentRef<typeof MyWidget>; "#)
    );
}

#[test]
fn external_kebab_case_component_refs_use_the_declared_template_binding() {
    let options = VirtualTsOptions {
        external_template_bindings: vec!["NuxtLink".into()],
        ..VirtualTsOptions::default()
    };
    assert_eq!(
        registry_with_summary_and_options(
            "import { useTemplateRef } from 'vue';\nconst link = useTemplateRef('link')",
            r#"<nuxt-link ref="link" />"#,
            &Croquis::default(),
            &options,
        )
        .map(|registry| registry.body)
        .as_deref(),
        Some(r#" "link": __VizeTemplateComponentRef<typeof NuxtLink>; "#)
    );
}

#[test]
fn unresolved_component_refs_keep_the_existing_any_component_fallback() {
    let registry = registry_of(r#"<MissingThing ref="fallback" />"#)
        .expect("unresolved components are still registered through the setup fallback");
    assert_eq!(
        registry.as_str(),
        r#" "fallback": __VizeTemplateComponentRef<typeof MissingThing>; "#
    );
}

#[test]
fn conditional_branches_still_register() {
    assert_eq!(
        registry_of(r#"<div v-if="a" ref="only" /><section v-else><p ref="deep" /></section>"#)
            .as_deref(),
        Some(r#" "only": __VizeDomElement<"div">; "deep": __VizeDomElement<"p">; "#)
    );
}
