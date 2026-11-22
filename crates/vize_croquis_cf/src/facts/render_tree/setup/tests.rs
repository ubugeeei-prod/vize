#![expect(
    clippy::unwrap_used,
    reason = "fixtures check source admission and registry membership"
)]
use super::*;
use crate::facts::{component_usage_targets, render_tree::edges_of};
use davinci_harness::alloc::{self, CountingAllocator};
use std::alloc::System;
use vize_croquis::{Croquis, Drawer, DrawerOptions};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::{NativeSfcObservation, lower_sfc_native};

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

mod corpus;
mod profile;

fn observe<'a>(arena: &'a Allocator, source: &'a str) -> NativeSfcObservation<'a> {
    lower_sfc_native(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    )
}

fn analysis(path: &Path, source: &str) -> Croquis {
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    if path.extension().is_some_and(|ext| ext == "vue") {
        if let Ok(sfc) = vize_croquis::sfc::parse_sfc(source, Default::default()) {
            if let Some(setup) = sfc.script_setup {
                drawer.draw_script_setup(&setup.content);
            }
            if let Some(script) = sfc.script {
                drawer.draw_script_plain(&script.content);
            }
            if let Some(template) = sfc.template {
                let arena = Allocator::new();
                let (root, _) = vize_armature::parse(&arena, &template.content);
                drawer.draw_template(&root);
            }
        }
    } else {
        drawer.draw_script_plain(source);
    }
    drawer.finish()
}

fn register(registry: &mut ModuleRegistry, path: &Path, source: &str) -> FileId {
    registry.register(path, source, analysis(path, source)).0
}

fn original(registry: &ModuleRegistry, path: &Path) -> SetupRenderProducts {
    let entry = registry.get_by_path(path).unwrap();
    SetupRenderProducts {
        edges: edges_of(registry, entry),
        targets: component_usage_targets(registry, entry.id),
    }
}

const PARENT: &str = "<script setup>import Box from './Box.vue'; import { Footer as Foot } from './ui';</script>\r\n<template>🪶<Box><Foot/></Box><Foot/><Box/></template>";

#[test]
fn nested_aliases_and_re_exports_preserve_complete_sites_and_dependency_order() {
    let arena = Allocator::new();
    let owner = observe(&arena, PARENT);
    let imports = owner.setup_component_imports().unwrap();
    let path = Path::new("/project/Parent.vue");
    let mut registry = ModuleRegistry::new();
    let (caller, input) = registry
        .register_setup_caller(path, analysis(path, PARENT), &imports)
        .unwrap();
    let box_id = register(
        &mut registry,
        Path::new("/project/Box.vue"),
        "<template>box</template>",
    );
    register(
        &mut registry,
        Path::new("/project/ui.ts"),
        "export { default as Footer } from './Footer.vue'",
    );
    let footer = register(
        &mut registry,
        Path::new("/project/Footer.vue"),
        "<template>footer</template>",
    );
    let actual = registry.setup_render_products(path, &input).unwrap();
    assert_eq!(actual, original(&registry, path));
    assert_eq!(actual.targets, [footer, box_id]);
    let parent_edge = RenderEdgeKey {
        caller,
        target: box_id,
        export_name: CompactString::new("default"),
    };
    let parent_site = &actual.edges[&parent_edge][0];
    let text = imports.template_span().slice(PARENT);
    assert_eq!(
        &text[parent_site.start as usize..parent_site.end as usize],
        "<Box>"
    );
    assert_eq!(
        imports.sites()[0].element_span().slice(PARENT),
        "<Box><Foot/></Box>"
    );
    let key = RenderEdgeKey {
        caller,
        target: footer,
        export_name: CompactString::new("default"),
    };
    let sites = &actual.edges[&key];
    let template = imports.template_span().slice(PARENT);
    assert_eq!(
        sites
            .iter()
            .map(|site| &template[site.start as usize..site.end as usize])
            .collect::<Vec<_>>(),
        ["<Foot/>", "<Foot/>"]
    );
    registry.remove(Path::new("/project/Footer.vue"));
    assert_eq!(
        registry.setup_render_products(path, &input).unwrap(),
        original(&registry, path)
    );
}

#[test]
fn absent_stale_and_incomplete_registry_inputs_refuse_the_whole_request() {
    let path = Path::new("/project/Parent.vue");
    let mut registry = ModuleRegistry::new();
    let arena = Allocator::new();
    let owner = observe(&arena, PARENT);
    let imports = owner.setup_component_imports().unwrap();
    let (_, input) = registry
        .register_setup_caller(path, analysis(path, PARENT), &imports)
        .unwrap();
    assert!(matches!(
        registry.register_setup_caller(Path::new("/project/wrong.ts"), Croquis::new(), &imports),
        Err(SetupRenderRefusal::Caller)
    ));
    assert_eq!(registry.len(), 1);
    registry.remove(path);
    assert_eq!(
        registry.setup_render_products(path, &input),
        Err(SetupRenderRefusal::Caller)
    );
    register(&mut registry, path, "<template>changed</template>");
    assert_eq!(
        registry.setup_render_products(path, &input),
        Err(SetupRenderRefusal::SourceRevision)
    );
    let mut incomplete = analysis(path, PARENT);
    incomplete.note_used_component("OnlyKnownByName");
    registry.register(path, PARENT, incomplete);
    assert_eq!(
        registry.setup_render_products(path, &input),
        Err(SetupRenderRefusal::UsageCoverage)
    );
    let mut duplicate = analysis(path, PARENT);
    duplicate.component_usages[1] = duplicate.component_usages[0].clone();
    registry.register(path, PARENT, duplicate);
    assert_eq!(
        registry.setup_render_products(path, &input),
        Err(SetupRenderRefusal::UsageCoverage)
    );
    let different_module = PARENT.replace("'./Box.vue'", "'./Not.vue'");
    registry.register(path, PARENT, analysis(path, &different_module));
    assert_eq!(
        registry.setup_render_products(path, &input),
        Err(SetupRenderRefusal::ImportIdentity)
    );
    registry.clear();
    register(&mut registry, Path::new("/project/Replacement.vue"), PARENT);
    assert_eq!(
        registry.setup_render_products(path, &input),
        Err(SetupRenderRefusal::Caller)
    );
}

#[test]
fn absent_module_uses_only_the_current_unique_component_name() {
    let source =
        "<script setup>import Card from './missing.vue'</script><template><Card/></template>";
    let path = Path::new("/project/App.vue");
    let arena = Allocator::new();
    let owner = observe(&arena, source);
    let imports = owner.setup_component_imports().unwrap();
    let mut registry = ModuleRegistry::new();
    let (_, input) = registry
        .register_setup_caller(path, analysis(path, source), &imports)
        .unwrap();
    let first = Path::new("/project/one/Card.vue");
    let second = Path::new("/project/two/Card.vue");
    let empty = registry.setup_render_products(path, &input).unwrap();
    assert_eq!(empty, original(&registry, path));
    assert!(empty.edges.is_empty());
    let card = register(&mut registry, first, "<template/>");
    let unique = registry.setup_render_products(path, &input).unwrap();
    assert_eq!(unique, original(&registry, path));
    assert_eq!(unique.targets, [card]);
    register(&mut registry, second, "<template/>");
    let ambiguous = registry.setup_render_products(path, &input).unwrap();
    assert_eq!(ambiguous, original(&registry, path));
    assert!(ambiguous.targets.is_empty());
    registry.remove(second);
    assert_eq!(
        registry.setup_render_products(path, &input).unwrap(),
        unique
    );
}

#[test]
fn empty_requests_still_require_the_registered_revision() {
    let source = "<script setup>const value=1</script><template><div/></template>";
    let arena = Allocator::new();
    let owner = observe(&arena, source);
    let imports = owner.setup_component_imports().unwrap();
    let path = Path::new("/project/Empty.vue");
    let mut registry = ModuleRegistry::new();
    let (_, input) = registry
        .register_setup_caller(path, analysis(path, source), &imports)
        .unwrap();
    assert_eq!(
        registry.setup_render_products(path, &input).unwrap(),
        original(&registry, path)
    );
    register(&mut registry, path, &source.replace("value=1", "value=2"));
    assert_eq!(
        registry.setup_render_products(path, &input),
        Err(SetupRenderRefusal::SourceRevision)
    );
}

#[test]
fn original_path_resolution_handles_relative_and_rooted_callers() {
    let source = "<script setup>const x=1</script><template><div/></template>";
    let arena = Allocator::new();
    let observation = observe(&arena, source);
    let imports = observation.setup_component_imports().unwrap();
    let relative = Path::new("App.vue");
    for mut registry in [
        ModuleRegistry::new(),
        ModuleRegistry::with_project_root("/project"),
    ] {
        let (_, input) = registry
            .register_setup_caller(relative, analysis(relative, source), &imports)
            .unwrap();
        assert_eq!(
            registry.setup_render_products(relative, &input).unwrap(),
            original(&registry, relative)
        );
        if registry.project_root().is_some() {
            let absolute = Path::new("/project/App.vue");
            assert_eq!(
                registry.setup_render_products(absolute, &input).unwrap(),
                original(&registry, absolute)
            );
        }
    }
}
