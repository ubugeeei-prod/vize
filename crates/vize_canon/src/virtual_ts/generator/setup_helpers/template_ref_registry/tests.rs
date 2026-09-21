use vize_croquis::Croquis;
use vize_relief::BindingType;
use vize_s0::{CompactString, FxHashSet};

use super::super::super::template_record::TemplateRecord;
use crate::virtual_ts::types::{VirtualTsCheckOptions, VirtualTsOptions};

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
    registry_checked(
        script,
        template,
        (summary, options),
        (VirtualTsCheckOptions::default(), &TemplateRecord::default()),
    )
}

fn registry_checked(
    script: &str,
    template: &str,
    (summary, options): (&Croquis, &VirtualTsOptions),
    checked: (VirtualTsCheckOptions, &TemplateRecord),
) -> Option<super::TemplateRefRegistry> {
    let allocator = vize_s0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    super::template_ref_registry(
        summary,
        options,
        Some(script),
        Some(&root),
        &FxHashSet::<CompactString>::default(),
        checked,
    )
}

// `$refs` reads the registry too, so a project that asks for it to be typed
// registers refs whether or not the script names `useTemplateRef`. A component
// ref is `null` there until its component is mounted; `useTemplateRef` adds
// that itself, and a ref in `v-for` holds it per target either way.
#[test]
fn dollar_refs_read_the_registry_with_unmounted_components() {
    let template = r#"<a ref="link" /><Child ref="child" /><Child v-for="i in 2" ref="many" />"#;
    for checks in [
        VirtualTsCheckOptions {
            infer_template_dollar_refs: true,
            ..Default::default()
        },
        VirtualTsCheckOptions {
            infer_component_dollar_refs: true,
            ..Default::default()
        },
    ] {
        let registry = registry_checked(
            "const label = 'hi'",
            template,
            (&Croquis::default(), &VirtualTsOptions::default()),
            (checks, &TemplateRecord::default()),
        )
        .unwrap();
        assert_eq!(
            (registry.body.as_str(), registry.dollar_body.as_str()),
            (
                r#" "link": __VizeDomElement<"a">; "child": __VizeTemplateComponentRef<typeof Child>; "many": (__VizeTemplateComponentRef<typeof Child> | null)[]; "#,
                r#" "link": __VizeDomElement<"a">; "child": __VizeTemplateComponentRef<typeof Child> | null; "many": (__VizeTemplateComponentRef<typeof Child> | null)[]; "#,
            )
        );
        assert!(!registry.includes_instantiated);
    }
}

// The usage the template instantiates is read back from the record the
// template scope returns, by the start of its element.
#[test]
fn an_instantiated_component_ref_reads_the_template_record() {
    let registry = registry_checked(
        "const child = useTemplateRef('child')",
        r#"<Child ref="declared" /><Child ref="child" :foo="1" />"#,
        (&Croquis::default(), &VirtualTsOptions::default()),
        (
            VirtualTsCheckOptions::default(),
            &TemplateRecord::instantiating(vec![24]),
        ),
    )
    .unwrap();
    assert_eq!(
        registry.body.as_str(),
        r#" "declared": __VizeTemplateComponentRef<typeof Child>; "child": __VizeTemplateRefInstance<typeof __vize_template.__vizeRefs, "24", typeof Child>; "#
    );
    assert!(registry.includes_instantiated);
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
fn a_dynamic_ref_binding_stays_out_of_the_registry() {
    assert_eq!(registry_of(r#"<div :ref="target" />"#), None);
}

/// Inside `v-for` the ref holds one target per iteration, and a name the
/// template registers more than once holds whichever of them is mounted.
#[test]
fn looped_refs_are_arrays_and_repeated_names_are_unions() {
    assert_eq!(
        registry_of(r#"<li v-for="it in xs" :key="it" ref="rows" />"#).as_deref(),
        Some(r#" "rows": __VizeDomElement<"li">[]; "#)
    );
    assert_eq!(
        registry_of(r#"<ul><li v-for="it in xs" :key="it"><a ref="links" /></li></ul>"#).as_deref(),
        Some(r#" "links": __VizeDomElement<"a">[]; "#)
    );
    assert_eq!(
        registry_of(r#"<div ref="dup" /><span ref="dup" />"#).as_deref(),
        Some(r#" "dup": __VizeDomElement<"div"> | __VizeDomElement<"span">; "#)
    );
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
