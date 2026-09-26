//! The authored L1 view must produce the same complete skeleton as Quirks.

use super::{authored_document_skeleton, authored_skeleton};
use crate::ir::TemplateSyntax;
use crate::markup::{L2Template, MarkupDocument};
use vize_l0::{Allocator, cstr};

fn compare(source: &str) {
    let allocator = Allocator::new();
    let lowered = L2Template::lower(&allocator, source);
    let markup = lowered.markup();
    let document = MarkupDocument::from_l2(&markup, TemplateSyntax::Vue);
    assert_eq!(
        authored_document_skeleton(&document),
        authored_skeleton(&allocator, source),
        "{source}"
    );
}

#[test]
fn authored_facade_matches_every_pair_and_triple_in_the_reuse_battery() {
    const TAGS: [&str; 24] = [
        "div",
        "p",
        "span",
        "a href=\"#\"",
        "button",
        "ul",
        "li",
        "dl",
        "dt",
        "table",
        "tbody",
        "tr",
        "td",
        "select",
        "option",
        "form",
        "h1",
        "svg",
        "math",
        "template",
        "textarea",
        "label",
        "MyCard",
        "b",
    ];
    const VOID: [&str; 3] = ["img", "input", "br"];
    fn name(tag: &str) -> &str {
        tag.split(' ').next().unwrap_or(tag)
    }
    let mut total = 0;
    for outer in TAGS {
        for inner in TAGS.iter().chain(VOID.iter()) {
            let body = if VOID.contains(inner) {
                cstr!("<{inner}>")
            } else {
                cstr!("<{inner}>t</{}>", name(inner))
            };
            for middle in ["", "span", "div", "tr"] {
                let source = if middle.is_empty() {
                    cstr!("<{outer}>{body}</{}>", name(outer))
                } else {
                    cstr!("<{outer}><{middle}>{body}</{middle}></{}>", name(outer))
                };
                compare(&source);
                total += 1;
            }
        }
    }
    assert_eq!(total, 2592);
}

#[test]
fn authored_facade_preserves_semantic_boundaries_and_attribute_facts() {
    for source in [
        r#"<svg><![CDATA[日本語😀]]><g><![CDATA[&amp;]]></g></svg>"#,
        r##"<a href="#"><span><a href="#">nested</a></span></a>"##,
        r#"<button><span><button>nested</button></span></button>"#,
        r#"<table><template v-for="row in rows"><tr><td>{{ row }}</td></tr></template></table>"#,
        r#"<Comp><template #footer><span><div>block</div></span></template><button>default</button></Comp>"#,
        r#"<Comp><template #[name]><div/></template><slot :name="name"><p>fallback</p></slot></Comp>"#,
        r#"<Transition><p><div>block</div></p></Transition><Teleport disabled><ul><div/></ul></Teleport>"#,
        r#"<TransitionGroup :tag="group"><div/></TransitionGroup><component :is="view"><p>slot</p></component>"#,
        r#"<svg><foreignObject><p><div>html</div></p></foreignObject></svg>"#,
        r##"<math><annotation-xml encoding="text&#47;html"><a href="#"><button>go</button></a></annotation-xml></math>"##,
        r#"<a :href="url"><button>uncertain</button></a><a v-bind="attrs"><button>uncertain</button></a>"#,
        r#"<span v-html="html"><div>unknown</div></span><span :textContent="text"><div>unknown</div></span>"#,
        r#"<div v-pre><template v-if="yes"><a :href="url">{{ raw }}</a></template></div>"#,
        r#"<span>&#x61;{{ text }}日本語😀</span><ul><li>one</li><li>two</li></ul>"#,
    ] {
        compare(source);
    }
}

fn element_trace(document: &MarkupDocument<'_>, authored: bool) -> Vec<vize_l0::CompactString> {
    let mut rows = Vec::new();
    let mut enter = |element: crate::markup::MarkupElement<'_>| {
        rows.push(cstr!(
            "{} {:?} {:?}",
            element.tag(),
            element.kind(),
            element.range()
        ));
        element.walk_attributes(&mut |attr| {
            rows.push(cstr!(
                "attribute {} {:?} {:?}",
                attr.name(),
                attr.value(),
                attr.range()
            ))
        });
        element.walk_bindings(&mut |binding| {
            let mut modifiers = Vec::new();
            binding.walk_modifiers(&mut |modifier| modifiers.push(modifier));
            rows.push(cstr!(
                "binding {:?} {:?} {:?} {:?} {:?} {:?}",
                binding.kind(),
                binding.arg_name(),
                binding.static_value(),
                binding.expression(),
                binding.range(),
                modifiers
            ));
        });
        element.walk_directives(&mut |directive| {
            rows.push(cstr!(
                "directive {} {:?} {:?}",
                directive.name(),
                directive.arg_name(),
                directive.range()
            ))
        });
        for name in ["for", "if", "slot"] {
            element.walk_authored_directive_ranges(name, &mut |range| {
                rows.push(cstr!("authored {name} {range:?}"))
            });
        }
        element.walk_opening_item_ranges(&mut |range| rows.push(cstr!("opening {range:?}")));
    };
    if authored {
        document.walk_authored_tree(&mut enter, &mut |_| {});
    } else {
        document.walk_tree(&mut enter, &mut |_| {});
    }
    rows
}

#[test]
fn authored_element_queries_match_the_non_repairing_template_facade() {
    use vize_armature::{Parser, ParserOptions, TemplateSyntaxMode};
    for source in [
        r#"<div v-pre v-bind:x.foo="a" :[name].camel="b" @click.once="save"><template v-for="x in xs" v-pre><span :href="url">{{ raw }}</span></template></div>"#,
        r#"<Comp :title="value"><template #[name]="scope"><slot :name="name" :value="scope"/></template></Comp>"#,
        r#"<table><template v-for="row in rows"><tr :key="row.id"><td encoding="text&#47;html">cell</td></tr></template></table>"#,
        r#"<template v-if="open"><a :href="url" @click.stop="save"/><component :is="view"/></template>"#,
    ] {
        let allocator = Allocator::new();
        let (root, _) = Parser::with_options_and_template_syntax(
            &allocator,
            source,
            ParserOptions::default(),
            TemplateSyntaxMode::Quirks,
        )
        .parse();
        let reference = MarkupDocument::new(&root, TemplateSyntax::Vue);
        let lowered = L2Template::lower(&allocator, source);
        let markup = lowered.markup();
        let document = MarkupDocument::from_l2(&markup, TemplateSyntax::Vue);
        assert_eq!(
            element_trace(&document, true),
            element_trace(&reference, false),
            "{source}"
        );
    }
}
