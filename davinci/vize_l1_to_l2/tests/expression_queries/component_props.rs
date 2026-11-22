//! Compare original component prop spellings, values, positions and usage order.
use super::{component_projection::draw, observe};
use vize_l0::{Allocator, Span};
use vize_l1_to_l2::native_file::SetupComponentImports;
use vize_l2::query::ComponentPropIssue;

fn compare(imports: &SetupComponentImports<'_, '_>) -> (usize, usize, usize) {
    let text = imports.source();
    let (original, start, _) = draw(text);
    let mut usage = vize_croquis::facts::component_usage_slice(&original).iter();
    let (mut sites, mut attrs, mut bound) = (0, 0, 0);
    imports
        .try_for_each_in_postorder(|site| {
            let other = usage.next().unwrap();
            assert_eq!(
                (site.tag(), site.element_span().start),
                (other.name.as_str(), other.start + start)
            );
            assert!(!other.has_spread_attrs);
            let mut expected = other.props.iter();
            site.visit_passed_props(|prop| {
                let old = expected.next().unwrap();
                assert_eq!(
                    (prop.name, prop.value, prop.span, prop.expression.is_some()),
                    (
                        old.name.as_str(),
                        old.value.as_deref(),
                        Span::new(start + old.start, start + old.end),
                        old.is_dynamic
                    ),
                    "{text}"
                );
                assert_eq!(prop.name_span.slice(text), prop.name);
                assert!(!old.name_is_dynamic);
                attrs += 1;
                bound += usize::from(prop.expression.is_some());
            })?;
            assert!(expected.next().is_none());
            sites += 1;
            Ok::<_, ComponentPropIssue>(())
        })
        .unwrap();
    assert!(usage.next().is_none());
    (sites, attrs, bound)
}

#[test]
fn nested_usages_yield_inner_first_with_interleaved_bound_and_literal_props() {
    let source = r#"<script setup lang="ts">import Card from './Card.vue'; const msg = 'hi';</script>
<template>😀<Card title="a&amp;b" :user-name="msg" flag><Card :title="msg" label="猫"/><Card/></Card><Card last/></template>"#;
    let arena = Allocator::new();
    let observed = observe(&arena, source);
    let imports = observed.setup_component_imports().unwrap();
    assert_eq!(compare(&imports), (4, 6, 2));
    let mut first = Vec::new();
    imports.sites()[0]
        .visit_passed_props(|prop| {
            first.push((
                prop.name,
                prop.value,
                prop.expression.map(|e| e.slice(source)),
            ));
        })
        .unwrap();
    assert_eq!(
        first,
        [
            ("title", Some("a&b"), None),
            ("user-name", Some("msg"), Some("msg")),
            ("flag", None, None)
        ]
    );
}

#[test]
fn unproven_prop_bindings_refuse_the_whole_response() {
    for attrs in [
        "v-bind='attrs'",
        ":[key]='value'",
        "v-model='value'",
        "@click='value'",
        ":x.prop='value'",
    ] {
        let source = format!(
            "<script setup>import Card from './Card.vue'; const key='name', value=1, attrs={{}};</script><template><Card/><Card {attrs}/></template>"
        );
        let arena = Allocator::new();
        let observed = observe(&arena, &source);
        if let Ok(imports) = observed.setup_component_imports() {
            let complete =
                imports.try_for_each_in_postorder(|site| site.visit_passed_props(|_| {}));
            assert!(
                matches!(complete, Err(ComponentPropIssue::Binding(_))),
                "{attrs}"
            );
        }
    }
}

#[test]
#[ignore = "compare complete original Croquis usage rows on all repository inputs"]
fn original_usage_rows() {
    let manifest =
        std::fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap())
            .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut total, mut invalid, mut absent, mut count, mut sites, mut attrs, mut bound) =
        (0, 0, 0, 0, 0, 0, 0);
    for file in manifest.split_terminator('\0') {
        total += 1;
        let data = std::fs::read(root.join(file)).unwrap();
        let Ok(text) = std::str::from_utf8(&data) else {
            invalid += 1;
            continue;
        };
        let arena = Allocator::new();
        let observed = observe(&arena, text);
        let Ok(imports) = observed.setup_component_imports() else {
            absent += 1;
            continue;
        };
        let (s, a, b) = compare(&imports);
        count += 1;
        sites += s;
        attrs += a;
        bound += b;
    }
    assert_eq!(total, invalid + absent + count);
    assert!(count > 0);
    eprintln!(
        "original_prop_rows total={total} invalid={invalid} outside={absent} admitted={count} sites={sites} props={attrs} bindings={bound}"
    );
}
