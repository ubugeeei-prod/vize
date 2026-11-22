//! Produce original import destinations and checker prop rows in one request.
use super::component_projection::{Site, retained_product};
use std::hint::black_box;
use vize_atelier_core::Parser;
use vize_croquis::{Drawer, DrawerOptions};
use vize_l0::Span;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::SetupComponentImports;
use vize_l1_to_l2::native_file::{NativeSfcObservation, lower_sfc_with_authored_bind_shorthand};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Products {
    targets: Vec<Site>,
    sites: Vec<SiteProps>,
}

#[derive(Debug, PartialEq, Eq)]
struct SiteProps {
    name: Span,
    props: Vec<(String, Option<String>, Span, bool)>,
}

fn products(
    owner: &NativeSfcObservation<'_>,
    imports: &SetupComponentImports<'_, '_>,
    joined: bool,
) -> Products {
    let targets = retained_product(imports);
    let mut sites = Vec::new();
    if joined {
        imports
            .try_for_each_in_postorder(|site| {
                let mut props = Vec::new();
                site.visit_passed_props(|p| {
                    props.push((
                        p.name.to_owned(),
                        p.value.map(str::to_owned),
                        p.span,
                        p.expression.is_some(),
                    ));
                })?;
                sites.push(SiteProps {
                    name: site.name_span(),
                    props,
                });
                Ok::<_, vize_l2::query::ComponentPropIssue>(())
            })
            .unwrap();
    } else {
        let block = owner.template().unwrap().block();
        let arena = Allocator::new();
        let (root, _) = Parser::new(&arena, block.source()).parse();
        let mut drawer = Drawer::with_options(DrawerOptions {
            track_usage: true,
            ..Default::default()
        });
        drawer.draw_template(&root);
        for site in vize_croquis::facts::component_usage_slice(drawer.croquis()) {
            let mut props = Vec::new();
            for p in &site.props {
                let span = Span::new(block.start() + p.start, block.start() + p.end);
                props.push((
                    p.name.to_string(),
                    p.value.as_ref().map(ToString::to_string),
                    span,
                    p.is_dynamic,
                ));
            }
            let first = block.start() + site.start + 1;
            let name = Span::new(first, first + site.name.len() as u32);
            sites.push(SiteProps { name, props });
        }
    }
    Products { targets, sites }
}

fn observe<'a>(arena: &'a Allocator, source: &'a str) -> NativeSfcObservation<'a> {
    lower_sfc_with_authored_bind_shorthand(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    )
}

pub(super) fn request_modes(mut check: impl FnMut(&str, bool, &dyn Fn(bool) -> Products)) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/lsp");
    let direct = "<script setup>import Card from './Card.vue';import Other from './Other.vue';const title=1, other=2;</script><template><Card :title v-bind:other><Other flag/><Card/></Card></template>";
    run_modes("direct", direct, &mut check);
    for name in [
        "same-name-rename/7994/Parent.vue.txt",
        "reactive-props-rename/7996/Parent.vue.txt",
        "reactive-props-rename/7996/supplemental/named-type-reactive/Parent.vue.txt",
        "same-name-rename/7994/supplemental/recursive-prop-roles/Parent.vue.txt",
    ] {
        let source = std::fs::read_to_string(root.join(name)).unwrap();
        run_modes(name, &source, &mut check);
    }
}

fn run_modes(
    label: &str,
    source: &str,
    check: &mut impl FnMut(&str, bool, &dyn Fn(bool) -> Products),
) {
    let arena = Allocator::new();
    let owner = observe(&arena, source);
    let retained = owner.setup_component_imports().unwrap();
    for cold in [false, true] {
        let run = |joined| {
            if cold {
                let a = Allocator::new();
                let owner = observe(&a, black_box(source));
                let imports = owner.setup_component_imports().unwrap();
                products(&owner, &imports, joined)
            } else {
                products(&owner, &retained, joined)
            }
        };
        assert_eq!(run(false), run(true));
        check(label, cold, &run);
    }
}
