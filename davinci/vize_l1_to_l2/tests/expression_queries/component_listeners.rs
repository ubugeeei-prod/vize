//! Compare every authored listener row and original handler operand with the shipped parser.
use super::{component_projection::draw, observe};
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::{
    NativeSfcObservation, SetupComponentImports, lower_sfc_with_listener_references,
};
use vize_l2::query::ComponentListenerIssue;

fn listeners<'a>(arena: &'a Allocator, source: &'a str) -> NativeSfcObservation<'a> {
    lower_sfc_with_listener_references(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    )
}

pub(super) fn compare(imports: &SetupComponentImports<'_, '_>) -> (usize, usize) {
    let source = imports.source();
    let (original, start, _) = draw(source);
    let mut usages = vize_croquis::facts::component_usage_slice(&original).iter();
    let (mut sites, mut events) = (0, 0);
    imports
        .try_for_each_in_postorder(|site| {
            let old = usages.next().unwrap();
            assert_eq!(
                (site.tag(), site.element_span().start),
                (old.name.as_str(), start + old.start),
                "{source}"
            );
            let mut expected = old.events.iter();
            site.visit_static_listeners(|row| {
                let event = expected.next().unwrap();
                assert!(!event.name_is_dynamic);
                assert_eq!(
                    (row.name, row.handler.map(|h| h.slice(source)), row.span),
                    (
                        event.name.as_str(),
                        event.handler.as_deref(),
                        Span::new(start + event.start, start + event.end)
                    ),
                    "{source}"
                );
                assert_eq!(row.name_span.slice(source), row.name);
                match (row.handler, row.handler_binding) {
                    (Some(span), Some(binding)) => {
                        assert!(binding.same_owner(site.binding()));
                        assert_eq!(
                            binding.declaration().unwrap().name.as_str(),
                            span.slice(source)
                        );
                    }
                    (None, None) => {}
                    _ => panic!("handler and declaration must belong to one reference"),
                }
                assert!(
                    row.modifiers
                        .iter()
                        .copied()
                        .eq(event.modifiers.iter().map(|m| m.as_str()))
                );
                events += 1;
            })?;
            assert!(expected.next().is_none());
            sites += 1;
            Ok::<_, ComponentListenerIssue>(())
        })
        .unwrap();
    assert!(usages.next().is_none());
    (sites, events)
}

#[test]
fn nested_component_listeners_keep_their_original_names_modifiers_and_handler_bindings() {
    let source = "<script setup lang='ts'>import Card from './Card.vue'; import { save, close as 閉じる } from './actions'; const title='unmodified';</script>\r\n<template>😀<Card :title @update:modelValue.once='save'><Card v-on:閉じる.stop.once='閉じる'/></Card><Card @done/></template>";
    let arena = Allocator::new();
    assert!(observe(&arena, source).setup_component_imports().is_err());
    let observation = listeners(&arena, source);
    let imports = observation.setup_component_imports().unwrap();
    assert_eq!(compare(&imports), (3, 3));
    let file = imports.sites()[0].binding().file();
    let model = vize_l2::query::Model::new(file);
    let mut uses = Vec::new();
    // An imported component is a typed general-File refusal. Its expression rows remain
    // attached to their exact original operations, scopes and setup declarations.
    for embed in observation.template().unwrap().embeds() {
        if embed.syntax.source().text() == "title" {
            continue;
        }
        let node = embed.node.unwrap();
        let expr = file.expression(node).unwrap().table().unwrap();
        assert_eq!(expr.occurrences().len(), 1);
        let span = embed.syntax.source().span();
        uses.push(span.slice(source));
        assert!(
            model
                .file()
                .binding(expr.occurrences()[0].binding)
                .is_some()
        );
    }
    assert_eq!(uses, ["save", "閉じる"]);
}

#[test]
fn inline_statements_dynamic_arguments_entities_and_incomplete_heads_refuse_the_whole_file() {
    for attr in [
        "@x='save()'",
        "@x='save; save'",
        "@x='$event'",
        "@x='arguments'",
        "@x='s&#97;ve'",
        "@[key]='save'",
        "v-on='handlers'",
        "@foo..stop='save'",
        "v-model='save'",
        "@x='bad +'",
    ] {
        let source = format!(
            "<script setup>import Card from './Card.vue'; const save=1;</script><template><Card @ok='save'/><Card {attr}/></template>"
        );
        let arena = Allocator::new();
        let observed = listeners(&arena, &source);
        assert!(observed.setup_component_imports().is_err(), "{attr}");
        assert!(!observed.template().unwrap().holes().is_empty(), "{attr}");
    }
}

#[test]
fn an_unsupported_setup_initializer_cannot_publish_a_partial_listener_file() {
    let source = "<script setup>import Card from './Card.vue'; const save=()=>1;</script><template><Card @save='save'/></template>";
    let arena = Allocator::new();
    let observed = listeners(&arena, source);
    assert!(observed.setup_component_imports().is_err());
    assert!(observed.template().unwrap().holes().is_empty());
    assert!(observed.admitted().is_none());
}

#[test]
#[ignore = "original Vue input manifest, full Croquis listener equality on every admitted owner"]
fn original_vue_files() {
    let list = std::fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap())
        .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (
        mut total,
        mut invalid,
        mut outside,
        mut admitted,
        mut components,
        mut with_events,
        mut events,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for file in list.split_terminator('\0') {
        total += 1;
        let bytes = std::fs::read(root.join(file)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        let arena = Allocator::new();
        let observed = listeners(&arena, source);
        let Ok(imports) = observed.setup_component_imports() else {
            outside += 1;
            continue;
        };
        let (sites, rows) = compare(&imports);
        admitted += 1;
        components += sites;
        with_events += usize::from(rows > 0);
        events += rows;
    }
    assert_eq!(total, invalid + outside + admitted);
    assert!(admitted > 0);
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_LISTENERS").is_some() {
        assert!(with_events >= 12 && events >= 13);
    }
    eprintln!(
        "original_component_listeners files={total} invalid={invalid} outside={outside} admitted={admitted} components={components} listener_files={with_events} listeners={events}"
    );
}
